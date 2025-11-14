# LazyDiff

A terminal-based diff viewer written in Rust that makes it easier to compare
files. The tool not only shows you nicely formatted diffs, it allows you to
copy the patch content to the clipboard or export it as a file.

## Getting Started

### Installation

You can install lazydiff using Cargo:

```bash
cargo install lazydiff
```

### Prerequisites

- Rust 1.70 or higher
- A terminal emulator with support for modern terminal features

### Building from Source

If you prefer to build from source:

```bash
git clone https://github.com/yourusername/lazydiff.git
cd lazydiff
cargo build --release
```

The binary will be available at `target/release/lazydiff`.

## Usage

### Basic Usage

Launch lazydiff with two files to compare:

```bash
lazydiff file1.txt file2.txt
```

Or start lazydiff and interactively select files:

```bash
lazydiff
```

You can also specify just the source file:

```bash
lazydiff source.txt
```

### Keyboard Shortcuts

**In Diff View:**
- `q` - Quit the application
- `s` - Select a new source file
- `t` - Select a new target file
- `c` - Copy diff to clipboard
- `e` - Export diff as a patch file
- `↑/↓` - Scroll through the diff

**In File Browser:**
- `↑/↓` - Navigate files and directories
- `Enter` - Select file or enter directory
- `Esc` or `q` - Cancel selection (or exit if no files selected)

### Features

- **Interactive File Browser**: Navigate your filesystem and select files to compare
- **Syntax Highlighting**: Color-coded diff output (green for additions, red for deletions)
- **Clipboard Integration**: Copy diffs directly to your clipboard with a single keypress
- **Patch Export**: Generate standard unified diff patch files
- **Intuitive Interface**: Clean, distraction-free TUI built with ratatui

## Documentation

For information on contributing to lazydiff, please see the [Contributing Guide](CONTRIBUTING.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.
