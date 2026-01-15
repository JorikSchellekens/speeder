use eframe::egui;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(target_os = "macos")]
#[allow(deprecated)]
mod hotkey {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // NSEventMask for key down events
    const NS_KEY_DOWN_MASK: u64 = 1 << 10;
    // Modifier flags
    const NS_COMMAND_KEY_MASK: u64 = 1 << 20;
    const NS_CONTROL_KEY_MASK: u64 = 1 << 18;

    pub fn setup_global_monitor(trigger: Arc<AtomicBool>) -> id {
        unsafe {
            let block = block::ConcreteBlock::new(move |event: id| {
                let flags: u64 = msg_send![event, modifierFlags];
                let has_cmd = (flags & NS_COMMAND_KEY_MASK) != 0;
                let has_ctrl = (flags & NS_CONTROL_KEY_MASK) != 0;

                if has_cmd && has_ctrl {
                    let chars: id = msg_send![event, charactersIgnoringModifiers];
                    if chars != nil {
                        let c_str: *const i8 = msg_send![chars, UTF8String];
                        if !c_str.is_null() {
                            let s = std::ffi::CStr::from_ptr(c_str).to_string_lossy();
                            if s.to_lowercase() == "r" {
                                println!("Hotkey detected: Cmd+Control+R");
                                trigger.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                }
            });
            let block = block.copy();

            let cls = class!(NSEvent);
            let monitor: id = msg_send![cls, addGlobalMonitorForEventsMatchingMask:NS_KEY_DOWN_MASK handler:&*block];

            // Keep the block alive
            std::mem::forget(block);

            monitor
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
}

impl SpeedReaderApp {
    fn new(trigger_flag: Arc<AtomicBool>, config: Config) -> Self {
        Self {
            engine: None,
            config,
            trigger_flag,
            reading_active: false,
            paused: false,
            window_visible: true, // Start visible, will hide on first update
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
                        self.config.speed.warmup_seconds,
                    ));
                    self.reading_active = true;
                    // Window visibility will be handled in update()

                    println!("Started reading {} words", text.split_whitespace().count());
                }
            }
        }
    }

    fn stop_reading(&mut self, _ctx: &egui::Context) {
        self.engine = None;
        self.reading_active = false;
        self.paused = false;
        // Window visibility will be handled in update()
        println!("Reading stopped, waiting for next trigger...");
    }
}

impl eframe::App for SpeedReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for trigger from hotkey listener
        if self.trigger_flag.swap(false, Ordering::Relaxed) && !self.reading_active {
            println!("Hotkey trigger received, starting reading...");
            self.start_reading(ctx);
        }

        // If not reading, hide window (only once) and wait
        if !self.reading_active {
            if self.window_visible {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.window_visible = false;
                println!("Window hidden");
            }
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }

        // Ensure window is visible during reading (only send command once)
        if !self.window_visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            self.window_visible = true;
            println!("Window shown");
        }

        // Handle keyboard input - collect actions first, then apply
        let mut should_toggle_pause = false;
        let mut should_stop = false;
        let mut speed_delta: i32 = 0;

        ctx.input(|i| {
            // Check all keys that were pressed this frame
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, .. } = event {
                    match key {
                        egui::Key::Space => {
                            println!("Space pressed");
                            should_toggle_pause = true;
                        }
                        egui::Key::Escape => {
                            println!("Escape pressed");
                            should_stop = true;
                        }
                        egui::Key::ArrowUp => {
                            println!("Arrow up pressed");
                            speed_delta += 50;
                        }
                        egui::Key::ArrowDown => {
                            println!("Arrow down pressed");
                            speed_delta -= 50;
                        }
                        _ => {}
                    }
                }
            }
        });

        // Apply actions after input processing
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
                }
            }
        }

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
        let word_parts: Option<(String, char, String)> = if let Some(engine) = &mut self.engine {
            engine.update().map(|w| {
                let (before, focus, after) = w.get_parts();
                (before, focus, after)
            })
        } else {
            None
        };

        let (progress, current_wpm) = if let Some(engine) = &self.engine {
            (engine.get_progress(), engine.get_current_wpm())
        } else {
            (0.0, 0)
        };

        // Main reading interface with rounded corners and border
        egui::CentralPanel::default()
            .frame(egui::Frame::none())  // Transparent panel background
            .show(ctx, |ui| {
                // Draw rounded rect background manually for proper clipping
                let panel_rect = ui.available_rect_before_wrap();
                let rounding = egui::Rounding::same(16.0);

                ui.painter().rect(
                    panel_rect,
                    rounding,
                    bg_color,
                    egui::Stroke::new(2.0, border_color),
                );

                // Content area with padding
                let content_rect = panel_rect.shrink(24.0);
                let mut content_ui = ui.child_ui(content_rect, egui::Layout::centered_and_justified(egui::Direction::TopDown), None);

                content_ui.vertical_centered(|ui| {
                    // Center vertically
                    let available_h = ui.available_height();
                    ui.add_space((available_h - 50.0) / 2.0);

                    if let Some((before, focus, after)) = &word_parts {
                        // Display word with ORP centered using a single formatted string
                        let font_size = 40.0;

                        ui.horizontal(|ui| {
                            ui.add_space((ui.available_width() - 650.0).max(0.0) / 2.0);

                            // Right-align "before" part
                            ui.label(
                                egui::RichText::new(format!("{:>12}", before))
                                    .size(font_size)
                                    .color(text_color)
                                    .monospace(),
                            );

                            // Focus character (highlighted)
                            ui.label(
                                egui::RichText::new(focus.to_string())
                                    .size(font_size)
                                    .color(focus_color)
                                    .monospace()
                                    .strong(),
                            );

                            // Left-align "after" part
                            ui.label(
                                egui::RichText::new(format!("{:<12}", after))
                                    .size(font_size)
                                    .color(text_color)
                                    .monospace(),
                            );
                        });
                    }

                    // Show progress and controls only when paused
                    if self.paused {
                        ui.add_space(15.0);

                        let progress_bar = egui::ProgressBar::new(progress)
                            .desired_width(280.0)
                            .desired_height(4.0)
                            .fill(focus_color.linear_multiply(0.7));
                        ui.add(progress_bar);

                        ui.add_space(6.0);

                        ui.label(
                            egui::RichText::new(format!("{} WPM  -  PAUSED", current_wpm))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(100, 100, 110)),
                        );
                    }
                });

                ctx.request_repaint();
            });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Load configuration
    let config = Config::load().unwrap_or_default();

    println!("Speed Reader running in background");
    println!("Global hotkey: Cmd+Control+R to read clipboard text");
    println!("");
    println!("Controls during reading:");
    println!("  Space: Pause/Resume");
    println!("  Up/Down: Adjust speed");
    println!("  ESC: Stop reading");

    // Shared flag for hotkey trigger
    let trigger_flag = Arc::new(AtomicBool::new(false));

    // Set up global hotkey monitor using macOS native APIs
    #[cfg(target_os = "macos")]
    let _monitor = hotkey::setup_global_monitor(Arc::clone(&trigger_flag));

    #[cfg(not(target_os = "macos"))]
    println!("Warning: Global hotkeys only supported on macOS");

    // Run the GUI app with wgpu backend (avoids OpenGL threading issues)
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 140.0])  // Wider for 25 chars + padding
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_visible(true), // Start visible, will hide on first update
        ..Default::default()
    };

    eframe::run_native(
        "Speed Reader",
        options,
        Box::new(move |_cc| Ok(Box::new(SpeedReaderApp::new(trigger_flag, config)))),
    )?;

    Ok(())
}