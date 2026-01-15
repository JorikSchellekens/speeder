use eframe::egui;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(target_os = "macos")]
mod hotkey {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::os::raw::c_void;

    // Carbon types and constants
    type OSStatus = i32;
    type EventHotKeyID = u32;
    type EventHotKeyRef = *mut c_void;

    const CMD_KEY: u32 = 1 << 8;  // cmdKey
    const CTRL_KEY: u32 = 1 << 12; // controlKey
    const K_VK_R: u32 = 15; // Virtual key code for 'R'

    #[repr(C)]
    struct EventTypeSpec {
        event_class: u32,
        event_kind: u32,
    }

    // Carbon event constants
    const K_EVENT_CLASS_KEYBOARD: u32 = 0x6b657962; // 'keyb'
    const K_EVENT_HOT_KEY_PRESSED: u32 = 5;

    #[repr(C)]
    struct HotKeyID {
        signature: u32,
        id: u32,
    }

    #[link(name = "Carbon", kind = "framework")]
    extern "C" {
        fn RegisterEventHotKey(
            key_code: u32,
            modifiers: u32,
            hot_key_id: HotKeyID,
            target: *mut c_void,
            options: u32,
            out_ref: *mut EventHotKeyRef,
        ) -> OSStatus;

        fn GetEventDispatcherTarget() -> *mut c_void;

        fn InstallEventHandler(
            target: *mut c_void,
            handler: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> OSStatus,
            num_types: u32,
            list: *const EventTypeSpec,
            user_data: *mut c_void,
            out_ref: *mut *mut c_void,
        ) -> OSStatus;
    }

    static mut TRIGGER_FLAG: Option<Arc<AtomicBool>> = None;

    extern "C" fn hotkey_handler(
        _next_handler: *mut c_void,
        _event: *mut c_void,
        _user_data: *mut c_void,
    ) -> OSStatus {
        unsafe {
            if let Some(ref trigger) = TRIGGER_FLAG {
                trigger.store(true, Ordering::Relaxed);
            }
        }
        0 // noErr
    }

    pub fn setup_global_hotkey(trigger: Arc<AtomicBool>) -> bool {
        unsafe {
            TRIGGER_FLAG = Some(trigger);

            let event_type = EventTypeSpec {
                event_class: K_EVENT_CLASS_KEYBOARD,
                event_kind: K_EVENT_HOT_KEY_PRESSED,
            };

            let mut handler_ref: *mut c_void = std::ptr::null_mut();
            let status = InstallEventHandler(
                GetEventDispatcherTarget(),
                hotkey_handler,
                1,
                &event_type,
                std::ptr::null_mut(),
                &mut handler_ref,
            );

            if status != 0 {
                eprintln!("Failed to install event handler: {}", status);
                return false;
            }

            let hot_key_id = HotKeyID {
                signature: 0x53504452, // 'SPDR'
                id: 1,
            };

            let mut hotkey_ref: EventHotKeyRef = std::ptr::null_mut();
            let status = RegisterEventHotKey(
                K_VK_R,
                CMD_KEY | CTRL_KEY,
                hot_key_id,
                GetEventDispatcherTarget(),
                0,
                &mut hotkey_ref,
            );

            if status != 0 {
                eprintln!("Failed to register hotkey: {}", status);
                return false;
            }

            true
        }
    }
}

mod config;
mod rsvp_engine;

use config::Config;
use rsvp_engine::RSVPEngine;

struct SpeedReaderApp {
    engine: Option<RSVPEngine>,
    config: Config,
    trigger_flag: Arc<AtomicBool>,
    reading_active: bool,
    paused: bool,
    window_visible: bool,
    last_word: Option<(String, char, String)>,
    progress_visible_until: Option<std::time::Instant>,
}

impl SpeedReaderApp {
    fn new(trigger_flag: Arc<AtomicBool>, config: Config) -> Self {
        Self {
            engine: None,
            config,
            trigger_flag,
            reading_active: false,
            paused: false,
            window_visible: true,
            last_word: None,
            progress_visible_until: None,
        }
    }

    fn start_reading(&mut self, _ctx: &egui::Context) {
        // Get clipboard content
        if let Ok(mut clipboard_ctx) = ClipboardContext::new() {
            if let Ok(text) = clipboard_ctx.get_contents() {
                if !text.is_empty() {
                    self.engine = Some(RSVPEngine::new(
                        &text,
                        self.config.speed.start_wpm,
                        self.config.speed.target_wpm,
                        self.config.speed.warmup_words,
                    ));
                    self.reading_active = true;
                }
            }
        }
    }

    fn stop_reading(&mut self, _ctx: &egui::Context) {
        self.engine = None;
        self.reading_active = false;
        self.paused = false;
        self.last_word = None;
        self.progress_visible_until = None;
    }
}

impl eframe::App for SpeedReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for trigger from hotkey listener
        if self.trigger_flag.swap(false, Ordering::Relaxed) && !self.reading_active {
            self.start_reading(ctx);
        }

        // If not reading, hide window and wait for hotkey
        if !self.reading_active {
            if self.window_visible {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.window_visible = false;
            }
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }

        // Ensure window is visible during reading
        if !self.window_visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            self.window_visible = true;
        }

        // Handle keyboard input - collect actions first, then apply
        let mut should_toggle_pause = false;
        let mut should_stop = false;
        let mut speed_delta: i32 = 0;

        let mut seek_delta: i32 = 0;
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, .. } = event {
                    match key {
                        egui::Key::Space => should_toggle_pause = true,
                        egui::Key::Escape => should_stop = true,
                        egui::Key::ArrowUp => speed_delta += 25,
                        egui::Key::ArrowDown => speed_delta -= 25,
                        egui::Key::ArrowLeft => seek_delta -= 1,
                        egui::Key::ArrowRight => seek_delta += 1,
                        _ => {}
                    }
                }
            }
        });

        // Apply seek and show progress bar for 1 second
        if seek_delta != 0 {
            if let Some(engine) = &mut self.engine {
                engine.seek(seek_delta);
                // Update displayed word after seek
                if let Some(word) = engine.get_current_word() {
                    let (before, focus, after) = word.get_parts();
                    self.last_word = Some((before, focus, after));
                }
                self.progress_visible_until = Some(std::time::Instant::now() + Duration::from_secs(1));
            }
        }

        // Apply speed changes from keyboard
        if speed_delta != 0 {
            if let Some(engine) = &mut self.engine {
                engine.adjust_speed(speed_delta);
            }
        }

        // Check if reading is finished
        if let Some(engine) = &self.engine {
            if engine.is_finished() {
                self.stop_reading(ctx);
                return;
            }
        }

        // Colors
        let bg_color = egui::Color32::from_rgb(20, 20, 25);
        let border_color = egui::Color32::from_rgb(60, 60, 70);
        let text_color = egui::Color32::from_rgb(200, 200, 210);
        let focus_color = egui::Color32::from_rgb(255, 100, 100);

        // Get word data and progress before UI rendering
        if let Some(engine) = &mut self.engine {
            if let Some(word) = engine.update() {
                let (before, focus, after) = word.get_parts();
                self.last_word = Some((before, focus, after));
            }
        }
        let word_parts = &self.last_word;

        let (progress, current_wpm) = if let Some(engine) = &self.engine {
            (engine.get_progress(), engine.get_current_wpm())
        } else {
            (0.0, 0)
        };

        // Apply keyboard actions
        if should_stop {
            self.stop_reading(ctx);
            return;
        }

        if should_toggle_pause {
            if let Some(engine) = &mut self.engine {
                self.paused = !self.paused;
                if self.paused {
                    engine.pause();
                } else {
                    engine.resume();
                    self.progress_visible_until = None;
                }
            }
        }

        // Make egui background fully transparent
        ctx.set_visuals(egui::Visuals {
            window_fill: egui::Color32::TRANSPARENT,
            panel_fill: egui::Color32::TRANSPARENT,
            ..egui::Visuals::dark()
        });

        // Main reading interface
        let paused = self.paused;
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();

                // Draw rounded background
                ui.painter().rect_filled(
                    rect,
                    egui::Rounding::same(12.0),
                    bg_color,
                );

                // Draw subtle border
                ui.painter().rect_stroke(
                    rect,
                    egui::Rounding::same(12.0),
                    egui::Stroke::new(1.0, border_color),
                );

                // Center the word display
                ui.vertical_centered(|ui| {
                    ui.add_space((rect.height() - 45.0) / 2.0);

                    if let Some((before, focus, after)) = word_parts {
                        let font_size = 34.0;

                        ui.horizontal(|ui| {
                            ui.add_space((ui.available_width() - 580.0).max(0.0) / 2.0);

                            ui.label(
                                egui::RichText::new(format!("{:>12}", before))
                                    .size(font_size)
                                    .color(text_color)
                                    .monospace(),
                            );

                            ui.label(
                                egui::RichText::new(focus.to_string())
                                    .size(font_size)
                                    .color(focus_color)
                                    .monospace()
                                    .strong(),
                            );

                            ui.label(
                                egui::RichText::new(format!("{:<12}", after))
                                    .size(font_size)
                                    .color(text_color)
                                    .monospace(),
                            );
                        });
                    }
                });

                // Slim progress bar at the bottom when paused or recently scrolled
                let show_bar = paused || self.progress_visible_until.map(|t| std::time::Instant::now() < t).unwrap_or(false);
                if show_bar {
                    let bar_height = 2.0;
                    let bar_margin = 12.0;
                    let bar_y = rect.bottom() - bar_height - 8.0;
                    let bar_width = rect.width() - (bar_margin * 2.0);

                    // Background track
                    let track_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.left() + bar_margin, bar_y),
                        egui::vec2(bar_width, bar_height),
                    );
                    ui.painter().rect_filled(
                        track_rect,
                        egui::Rounding::same(1.5),
                        egui::Color32::from_rgb(40, 40, 50),
                    );

                    // Progress fill
                    let fill_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.left() + bar_margin, bar_y),
                        egui::vec2(bar_width * progress, bar_height),
                    );
                    ui.painter().rect_filled(
                        fill_rect,
                        egui::Rounding::same(1.5),
                        focus_color.linear_multiply(0.8),
                    );
                }

            });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Load configuration
    let config = Config::load().unwrap_or_default();

    println!("Speed Reader - Cmd+Control+R to start, ESC to stop");

    // Shared flag for hotkey trigger
    let trigger_flag = Arc::new(AtomicBool::new(false));

    // Set up global hotkey using Carbon API (doesn't block window focus)
    #[cfg(target_os = "macos")]
    hotkey::setup_global_hotkey(Arc::clone(&trigger_flag));

    // Run the GUI app with transparent background
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 90.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Speed Reader",
        options,
        Box::new(move |_cc| Ok(Box::new(SpeedReaderApp::new(trigger_flag, config)))),
    )?;

    Ok(())
}