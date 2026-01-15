#!/bin/bash

# Create a simple test binary
cat > src/bin/test.rs << 'EOF'
use speeder::rsvp_engine::RSVPEngine;

fn main() {
    println!("Testing RSVP Engine...\n");

    let test_text = "This is a test of the Rapid Serial Visual Presentation system. \
                     It displays words one at a time with an optimal recognition point.";

    let mut engine = RSVPEngine::new(test_text, 200, 400, 5);

    println!("Starting at {} WPM, targeting {} WPM", 200, 400);
    println!("Press Ctrl+C to stop\n");

    loop {
        if let Some(word) = engine.update() {
            let (before, focus, after) = word.get_parts();

            // Clear line and print word with ORP marker
            print!("\r{:>20}{}{:<20} ", before, focus, after);
            print!(" [{}ms]", word.display_time.as_millis());

            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            std::thread::sleep(word.display_time);
        } else if engine.is_finished() {
            println!("\n\nFinished reading!");
            break;
        }
    }
}
EOF

# Add lib.rs to expose modules
cat > src/lib.rs << 'EOF'
pub mod rsvp_engine;
pub mod config;
EOF

echo "Running console test..."
cargo run --bin test
