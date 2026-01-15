#!/bin/bash

set -e

echo "Building Speed Reader for macOS..."

# Build in release mode
cargo build --release

echo "Build complete!"
echo ""
echo "To run:"
echo "  ./target/release/speed-reader"
echo ""
echo "Usage:"
echo "  1. Copy text to clipboard"
echo "  2. Press Cmd+R to start reading"
echo "  3. Space to pause, arrows to adjust speed"
echo "  4. ESC to stop reading current text"
echo "  5. Q to quit application"