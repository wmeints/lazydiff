# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

lazydiff is a terminal-based diff viewer written in Rust. The application allows users to:

- Browse and select two files from subdirectories relative to the current directory
- Select files using command line options (`lazydiff [file-1] [file-2]`)
- View diffs with syntax highlighting (red for removed lines, - indicators for removed lines, green for added lines, + indicators for added lines)
- Copy diffs to clipboard (press `c`)
- Export diffs as patch files (press `e`)
- Close the application (press `q`)
- Select source files (press `s`)
- Select target files (press `t`)

## Technology Stack

- **UI Framework**: [ratatui](https://ratatui.rs/) - Terminal UI library
- **Diff Engine**: [similar](https://docs.rs/similar/) - Text diffing library
- **Terminal Control**: [crossterm](https://docs.rs/crossterm/) - Cross-platform terminal manipulation
- **Commandline parser**: [clap](https://docs.rs/clap/latest/clap/)

## Development Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Run in release mode (optimized)
cargo run --release

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

## Architecture Notes

The application follows a typical TUI architecture pattern:
- Event loop handling keyboard input
- State management for file browser, diff view, and active UI mode
- Separate rendering logic for different UI components (file browser, diff viewer)
- Integration with system clipboard for copy functionality
- File I/O for patch export functionality
