# Contributing to LazyDiff

Thank you for your interest in contributing to LazyDiff! This guide will help
you get started with development.

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Git
- A terminal emulator

### Setting Up Your Development Environment

1. **Fork and clone the repository:**

```bash
git clone https://github.com/yourusername/lazydiff.git
cd lazydiff
```

2. **Build the project:**

```bash
cargo build
```

3. **Run the application:**

```bash
cargo run
```

You can also pass arguments to test specific functionality:

```bash
cargo run -- file1.txt file2.txt
```

## Building the Project

### Development Build

For faster compilation during development:

```bash
cargo build
```

The binary will be available at `target/debug/lazydiff`.

### Release Build

For optimized builds:

```bash
cargo build --release
```

The binary will be available at `target/release/lazydiff`.

### Checking Code Without Building

To quickly check if your code compiles without producing binaries:

```bash
cargo check
```

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Unit Tests Only

```bash
cargo test --lib
```

### Run Integration Tests Only

```bash
cargo test --test cli
```

### Run Tests with Output

To see test output (useful for debugging):

```bash
cargo test -- --nocapture
```

### Run a Specific Test

```bash
cargo test test_name
```

## Code Quality

### Format Code

Before submitting a PR, ensure your code is formatted:

```bash
cargo fmt
```

### Run Linter

Check for common mistakes and style issues:

```bash
cargo clippy
```

Fix any warnings or errors that clippy reports.

## Project Structure

LazyDiff is organized into several modules, each with a specific responsibility:

```
lazydiff/
├── src/
│   ├── main.rs       # Entry point, CLI parsing, terminal initialization
│   ├── app.rs        # Core application logic, event loop, state management
│   ├── browser.rs    # File browser functionality and navigation
│   ├── diff.rs       # Diff generation, patch formatting, clipboard/export
│   └── ui.rs         # Terminal UI rendering components
├── tests/
│   └── cli.rs        # Integration tests for CLI functionality
├── Cargo.toml        # Project metadata and dependencies
├── CLAUDE.md         # Project guidance for AI assistants
└── README.md         # User-facing documentation
```

### Module Responsibilities

#### `main.rs`
- Command-line argument parsing using clap
- Terminal setup and cleanup
- App initialization based on CLI arguments
- Minimal entry point (~98 lines)

#### `app.rs`
- `App` struct containing application state
- `AppMode` enum for tracking current mode (DiffView, SelectingSource, SelectingTarget)
- Main event loop in `run_app()`
- Event handlers: `handle_diffview_input()`, `handle_browser_input()`, `handle_file_selection()`
- Unit tests for application logic

#### `browser.rs`
- `FileBrowser` struct for directory navigation
- Directory listing and sorting (directories first, then files)
- Navigation methods (move_up, move_down, enter_selected)
- Scroll management for viewport

#### `diff.rs`
- `DiffLine` struct representing individual diff lines
- `generate_diff()` - Creates diff from file contents using the `similar` crate
- `generate_patch()` - Formats diff as unified patch
- `copy_to_clipboard()` - Clipboard integration via `arboard`
- `export_to_file()` - Exports patch to timestamped file
- `validate_file()` - File validation helper
- Unit tests for diff operations

#### `ui.rs`
- Rendering functions for all UI components
- `render_ui()` - Main rendering coordinator
- `render_header()` - File header display
- `render_diff_view()` - Diff content with syntax highlighting
- `render_file_browser()` - File browser UI
- `render_status_bar()` - Status and help text

### Key Dependencies

- **ratatui** - Terminal user interface framework
- **crossterm** - Cross-platform terminal manipulation
- **similar** - Text diffing algorithm
- **arboard** - Clipboard access
- **clap** - Command-line argument parsing

## Making Changes

### Workflow

1. Create a new branch for your feature or fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, following the existing code style

3. Add tests for new functionality

4. Run tests and linting:
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

5. Commit your changes with a clear message:
   ```bash
   git commit -m "Add feature: description of your changes"
   ```

6. Push to your fork and create a pull request

### Code Style Guidelines

- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Keep functions focused and single-purpose
- Add comments for complex logic
- Write descriptive commit messages
- Include tests for new features or bug fixes
- Ensure all clippy warnings are addressed

### Testing Guidelines

- Write unit tests in the same file as the code being tested (in `#[cfg(test)]` modules)
- Use descriptive test names that explain what is being tested
- Test both success and error cases
- For tests that use the clipboard, acquire the `CLIPBOARD_LOCK` mutex to prevent race conditions
- Clean up test files and resources in test cleanup functions

## Reporting Issues

When reporting bugs, please include:

- LazyDiff version (`lazydiff --version`)
- Operating system and version
- Steps to reproduce the issue
- Expected vs. actual behavior
- Any relevant error messages

## Questions?

If you have questions about contributing, feel free to open an issue for discussion.

Thank you for contributing to LazyDiff!
