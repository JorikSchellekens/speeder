# Speed Reader - RSVP System for macOS

A Rust-based Rapid Serial Visual Presentation (RSVP) speed reading system with Optimal Recognition Point (ORP) for macOS.

## Features

- **RSVP Display**: Shows one word at a time with optimal focus point
- **ORP (Optimal Recognition Point)**: Calculates the best fixation point for each word
- **Adaptive Speed**: Warm-up period from start speed to target speed
- **Clipboard Integration**: Reads text from clipboard
- **Visual Focus**: Black background with red focus letter at ORP
- **Keyboard Controls**: Full control over reading experience
- **Real-time Speed Adjustment**: Change reading speed on the fly

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd speed-reader

# Quick install
./install.sh

# Or build only
./build.sh
```

## Configuration

Configuration file: `~/Library/Application Support/speed-reader/config.toml`

Default settings:
- Start speed: 200 WPM
- Target speed: 400 WPM
- Warm-up time: 30 seconds
- Font size: 48px

Edit `config.toml` in the project directory to customize before first run.

## Usage

### Starting the Application
```bash
# Run from terminal
speed-reader

# Or double-click SpeedReader.command on Desktop (after install)
# Or run directly
./target/release/speed-reader
```

### Reading Workflow
1. Copy any text to clipboard
2. Press `Cmd+R` to start reading
3. Use controls below during reading

### Keyboard Controls
- `Cmd+R`: Start reading clipboard text
- `Space`: Pause/Resume
- `↑`: Increase speed by 10 WPM
- `↓`: Decrease speed by 10 WPM
- `R`: Restart current text
- `Escape`: Stop reading current text
- `Q`: Quit application

## How ORP Works

The Optimal Recognition Point is calculated based on word length:
- 1-3 letters: Focus on 1st letter
- 4-5 letters: Focus on 2nd letter
- 6-9 letters: Focus on 3rd letter
- 10-13 letters: Focus on 4th letter
- 14+ letters: Focus on 5th letter

The focus letter appears in red with proper alignment for optimal reading speed.

## Development

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run development version
cargo run

# Clean build artifacts
cargo clean
```

## Uninstall

```bash
# Remove command line shortcut
sudo rm /usr/local/bin/speed-reader

# Remove desktop shortcut
rm ~/Desktop/SpeedReader.command

# Remove configuration
rm -rf ~/Library/Application\ Support/speed-reader

# Remove project directory (optional)
rm -rf /path/to/speed-reader
```

## Dependencies

- macroquad: Cross-platform graphics and window management
- clipboard: Clipboard access
- serde/toml: Configuration management
- Standard Rust libraries for threading and synchronization

## Requirements

- macOS 10.12 or later
- Rust 1.70 or later
- Cargo package manager

## License

MIT