#!/bin/bash

set -e

echo "Installing Speed Reader..."

# Build the application
echo "Building application..."
./build.sh

# Create symlink in /usr/local/bin for easy access
echo "Creating command line shortcut..."
sudo ln -sf "$(pwd)/target/release/speed-reader" /usr/local/bin/speed-reader

# Create desktop entry for easy launching
DESKTOP_FILE="$HOME/Desktop/SpeedReader.command"
cat > "$DESKTOP_FILE" << EOF
#!/bin/bash
cd "$(pwd)"
./target/release/speed-reader
EOF
chmod +x "$DESKTOP_FILE"

echo ""
echo "Installation complete!"
echo ""
echo "You can now run Speed Reader in several ways:"
echo "  1. From terminal: speed-reader"
echo "  2. Double-click SpeedReader.command on your Desktop"
echo "  3. Run directly: ./target/release/speed-reader"
echo ""
echo "Usage:"
echo "  - Copy text to clipboard"
echo "  - Press Cmd+R to start reading"
echo "  - Press Space to pause/resume"
echo "  - Press Up/Down arrows to adjust speed"
echo "  - Press Escape to stop reading"
echo "  - Press Q to quit application"
echo ""
echo "Configuration file: ~/Library/Application Support/speed-reader/config.toml"
echo ""
echo "To uninstall:"
echo "  sudo rm /usr/local/bin/speed-reader"
echo "  rm ~/Desktop/SpeedReader.command"