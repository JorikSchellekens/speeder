#!/bin/bash

set -e

echo "Installing Speeder..."

# Build the application
echo "Building application..."
./build.sh

# Create symlink in /usr/local/bin for easy access
echo "Creating command line shortcut..."
sudo ln -sf "$(pwd)/target/release/speeder" /usr/local/bin/speeder

# Create desktop entry for easy launching
DESKTOP_FILE="$HOME/Desktop/Speeder.command"
cat > "$DESKTOP_FILE" << EOF
#!/bin/bash
cd "$(pwd)"
./target/release/speeder
EOF
chmod +x "$DESKTOP_FILE"

echo ""
echo "Installation complete!"
echo ""
echo "You can now run Speeder in several ways:"
echo "  1. From terminal: speeder"
echo "  2. Double-click Speeder.command on your Desktop"
echo "  3. Run directly: ./target/release/speeder"
echo ""
echo "Usage:"
echo "  - Copy text to clipboard (or select text)"
echo "  - Press Cmd+Control+R to start reading"
echo "  - Press Space to pause/resume"
echo "  - Press Up/Down arrows to adjust speed"
echo "  - Press Left/Right arrows to navigate"
echo "  - Press R to restart"
echo "  - Press Escape to stop reading"
echo ""
echo "Configuration file: ~/Library/Application Support/speeder/config.toml"
echo ""
echo "To uninstall:"
echo "  sudo rm /usr/local/bin/speeder"
echo "  rm ~/Desktop/Speeder.command"
