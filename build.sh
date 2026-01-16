#!/bin/bash

set -e

echo "Building Speeder for macOS..."

# Build in release mode
cargo build --release

# Create .app bundle
APP_NAME="Speeder"
APP_DIR="target/release/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "Creating app bundle..."

# Clean and create directory structure
rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binary
cp "target/release/speeder" "${MACOS_DIR}/${APP_NAME}"

# Create Info.plist
cat > "${CONTENTS_DIR}/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Speeder</string>
    <key>CFBundleDisplayName</key>
    <string>Speeder</string>
    <key>CFBundleIdentifier</key>
    <string>com.speeder.app</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>Speeder</string>
    <key>LSUIElement</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
EOF

echo "Build complete!"
echo ""
echo "App bundle created at: ${APP_DIR}"
echo ""
echo "To run:"
echo "  open target/release/Speeder.app"
echo "  # or"
echo "  ./target/release/speeder"
echo ""
echo "To install, run: ./install.sh"
