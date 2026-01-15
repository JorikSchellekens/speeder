use eframe::egui;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[cfg(target_os = "macos")]
mod hotkey {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::runtime::Object;
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

        // Handle keyboard input during reading
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                if let Some(engine) = &mut self.engine {
                    self.paused = !self.paused;
                    if self.paused {
                        engine.pause();
                    } else {
                        engine.resume();
                    }
                }
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                if let Some(engine) = &mut self.engine {
                    engine.adjust_speed(10);
                }
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                if let Some(engine) = &mut self.engine {
                    engine.adjust_speed(-10);
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                self.stop_reading(ctx);
                return;
            }
        });

        // Check if reading is finished
        if let Some(engine) = &self.engine {
            if engine.is_finished() {
                // Auto-stop after finished (with small delay)
                self.stop_reading(ctx);
                return;
            }
        }

        // Main reading interface
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                if let Some(engine) = &mut self.engine {
                    // Update and display word
                    if let Some(word) = engine.update() {
                        let (before, focus, after) = word.get_parts();

                        // Center the display
                        ui.vertical_centered(|ui| {
                            ui.add_space(ui.available_height() / 2.0 - 30.0);

                            ui.horizontal(|ui| {
                                // Add horizontal centering
                                let total_width = 600.0;
                                let side_margin = (ui.available_width() - total_width) / 2.0;
                                ui.add_space(side_margin.max(0.0));

                                // Display word parts with ORP alignment
                                ui.label(
                                    egui::RichText::new(format!("{:>20}", before))
                                        .size(48.0)
                                        .color(egui::Color32::LIGHT_GRAY)
                                        .monospace(),
                                );

                                ui.label(
                                    egui::RichText::new(focus.to_string())
                                        .size(48.0)
                                        .color(egui::Color32::RED)
                                        .monospace()
                                        .strong(),
                                );

                                ui.label(
                                    egui::RichText::new(format!("{:<20}", after))
                                        .size(48.0)
                                        .color(egui::Color32::LIGHT_GRAY)
                                        .monospace(),
                                );
                            });

                            // Visual guide line for ORP
                            let center_x = ui.available_width() / 2.0;
                            let line_top = ui.cursor().top() - 60.0;
                            let line_bottom = ui.cursor().top() + 20.0;

                            ui.painter().line_segment(
                                [
                                    egui::pos2(center_x, line_top),
                                    egui::pos2(center_x, line_bottom)
                                ],
                                egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                            );
                        });
                    }

                    // Progress bar and info at bottom
                    let progress = engine.get_progress();
                    let current_wpm = engine.get_current_wpm();

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(10.0);

                        // Progress bar
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .desired_width(ui.available_width() * 0.8)
                                .desired_height(4.0)
                        );

                        // WPM indicator
                        ui.label(
                            egui::RichText::new(format!("{} WPM", current_wpm))
                                .size(16.0)
                                .color(egui::Color32::GRAY),
                        );

                        // Controls hint
                        ui.label(
                            egui::RichText::new("Space: Pause | ↑↓: Speed | ESC: Stop")
                                .size(12.0)
                                .color(egui::Color32::DARK_GRAY),
                        );
                    });
                }

                // Request continuous updates during reading
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
            .with_inner_size([900.0, 400.0])
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