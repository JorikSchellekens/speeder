# Speeder - RSVP Speed Reader for macOS

A Rust-based Rapid Serial Visual Presentation (RSVP) speed reading system with Optimal Recognition Point (ORP) for macOS.

## Features

- **RSVP Display**: Shows one word at a time with optimal focus point
- **ORP (Optimal Recognition Point)**: Calculates the best fixation point for each word
- **Adaptive Speed**: Warm-up from 75% to target speed over configurable word count
- **Selection/Clipboard Integration**: Reads selected text or clipboard content
- **Visual Focus**: Dark background with red focus letter at ORP
- **Position Memory**: Remembers position when reopening the same text
- **Keyboard Controls**: Full control over reading experience
- **Real-time Speed Adjustment**: Change reading speed on the fly (persisted)

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd speeder

# Quick install
./install.sh

# Or build only
./build.sh
```

## Configuration

Configuration file: `~/Library/Application Support/speeder/config.toml`

Default settings:
- Target speed: 400 WPM (warmup starts at 75%)
- Warm-up words: 10

## Usage

### Starting the Application
```bash
# Run from terminal
speeder

# Or double-click Speeder.command on Desktop (after install)
# Or run directly
./target/release/speeder
```

### Reading Workflow
1. Select text in any app (or copy to clipboard)
2. Press `Cmd+Control+R` to start reading
3. Use controls below during reading

### Keyboard Controls
- `Cmd+Control+R`: Start reading selected/clipboard text
- `Space`: Pause/Resume
- `Up/Down`: Adjust speed by 25 WPM
- `Left/Right`: Navigate words
- `R`: Restart from beginning
- `Escape`: Stop reading

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
sudo rm /usr/local/bin/speeder

# Remove desktop shortcut
rm ~/Desktop/Speeder.command

# Remove configuration
rm -rf ~/Library/Application\ Support/speeder
```

## Dependencies

- eframe/egui: GUI framework
- clipboard: Clipboard access
- serde/toml: Configuration management
- Carbon framework: Global hotkey registration

## Requirements

- macOS 10.12 or later
- Rust 1.70 or later

## License

MIT
