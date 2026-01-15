#!/bin/bash

set -e

echo "Building Speeder for macOS..."

# Build in release mode
cargo build --release

echo "Build complete!"
echo ""
echo "To run:"
echo "  ./target/release/speeder"
echo ""
echo "Usage:"
echo "  1. Copy text to clipboard"
echo "  2. Press Cmd+Control+R to start reading"
echo "  3. Space to pause, arrows to adjust speed"
echo "  4. ESC to stop reading current text"
