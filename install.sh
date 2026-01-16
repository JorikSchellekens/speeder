#!/bin/bash

set -e

echo "Installing Speeder..."

# Build the application
echo "Building application..."
./build.sh

# Copy app bundle to /Applications
echo "Installing to /Applications..."
rm -rf "/Applications/Speeder.app"
cp -r "target/release/Speeder.app" "/Applications/"

# Create LaunchAgent for login startup
LAUNCH_AGENT_DIR="$HOME/Library/LaunchAgents"
LAUNCH_AGENT_FILE="$LAUNCH_AGENT_DIR/com.speeder.app.plist"

echo "Setting up login startup..."
mkdir -p "$LAUNCH_AGENT_DIR"

cat > "$LAUNCH_AGENT_FILE" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.speeder.app</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Applications/Speeder.app/Contents/MacOS/Speeder</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
EOF

# Load the LaunchAgent (start now if not already running)
launchctl unload "$LAUNCH_AGENT_FILE" 2>/dev/null || true
launchctl load "$LAUNCH_AGENT_FILE"

echo ""
echo "Installation complete!"
echo ""
echo "Speeder is now:"
echo "  - Installed at /Applications/Speeder.app"
echo "  - Set to start automatically at login"
echo "  - Running in the menubar (look for 'Speeder' text)"
echo ""
echo "Usage:"
echo "  - Select text in any app (or copy to clipboard)"
echo "  - Press Cmd+Control+R to start reading"
echo "  - Space to pause/resume"
echo "  - Up/Down arrows to adjust speed"
echo "  - Left/Right arrows to navigate"
echo "  - R to restart"
echo "  - Escape to stop"
echo ""
echo "To uninstall:"
echo "  launchctl unload ~/Library/LaunchAgents/com.speeder.app.plist"
echo "  rm ~/Library/LaunchAgents/com.speeder.app.plist"
echo "  rm -rf /Applications/Speeder.app"
echo "  rm -rf ~/Library/Application\\ Support/speeder"
