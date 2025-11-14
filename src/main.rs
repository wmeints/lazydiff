use arboard::Clipboard;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

/// A terminal-based diff viewer
#[derive(Parser)]
#[command(name = "lazydiff")]
#[command(version)]
#[command(about = "A terminal-based diff viewer", long_about = None)]
struct Cli {
    /// Source file to compare
    source: Option<String>,

    /// Target file to compare against
    target: Option<String>,
}

struct App {
    source_file: String,
    target_file: String,
    diff_lines: Vec<DiffLine>,
    scroll_offset: usize,
    status_message: Option<String>,
    clipboard: Clipboard,
}

#[derive(Clone)]
struct DiffLine {
    tag: ChangeTag,
    content: String,
}

impl App {
    fn new(source_file: String, target_file: String) -> Result<Self, Box<dyn std::error::Error>> {
        let source_content = fs::read_to_string(&source_file)?;
        let target_content = fs::read_to_string(&target_file)?;

        let diff = TextDiff::from_lines(&source_content, &target_content);
        let mut diff_lines = Vec::new();

        for change in diff.iter_all_changes() {
            let tag = change.tag();
            let content = change.value().to_string();

            // Split multi-line changes into separate lines
            for line in content.lines() {
                diff_lines.push(DiffLine {
                    tag,
                    content: line.to_string(),
                });
            }

            // Handle the case where the content ends with a newline
            if content.ends_with('\n') && !content.trim().is_empty() {
                // Already handled by lines()
            }
        }

        let clipboard = Clipboard::new()?;

        Ok(App {
            source_file,
            target_file,
            diff_lines,
            scroll_offset: 0,
            status_message: None,
            clipboard,
        })
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self, max_visible_lines: usize) {
        if self.scroll_offset + max_visible_lines < self.diff_lines.len() {
            self.scroll_offset += 1;
        }
    }

    fn generate_patch(&self) -> String {
        let mut patch = String::new();

        // Add patch header
        patch.push_str(&format!("--- {}\n", self.source_file));
        patch.push_str(&format!("+++ {}\n", self.target_file));

        // Add diff lines in unified format
        for diff_line in &self.diff_lines {
            let prefix = match diff_line.tag {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            patch.push_str(&format!("{}{}\n", prefix, diff_line.content));
        }

        patch
    }

    fn copy_to_clipboard(&mut self) -> Result<(), String> {
        let patch = self.generate_patch();

        self.clipboard.set_text(patch)
            .map_err(|e| format!("Failed to copy to clipboard: {}", e))
    }

    fn export_to_file(&self) -> Result<String, String> {
        let patch = self.generate_patch();

        // Generate filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Failed to get timestamp: {}", e))?
            .as_secs();

        let filename = format!("diff_{}.patch", timestamp);

        // Write patch to file (fs::write handles flushing automatically)
        fs::write(&filename, patch.as_bytes())
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        Ok(filename)
    }
}

fn validate_file(path: &str, file_type: &str) -> Result<(), String> {
    let file_path = Path::new(path);

    if !file_path.exists() {
        return Err(format!("{} file '{}' does not exist", file_type, path));
    }

    if !file_path.is_file() {
        return Err(format!("{} path '{}' is not a file", file_type, path));
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header with file names
                    Constraint::Min(0),    // Diff content
                    Constraint::Length(3), // Status bar
                ])
                .split(f.area());

            // Header with file names
            let header = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&app.source_file),
                    Span::raw("  "),
                    Span::styled("Target: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(&app.target_file),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).title("Files"));

            f.render_widget(header, chunks[0]);

            // Diff content
            let content_height = chunks[1].height.saturating_sub(2) as usize; // Subtract borders
            let visible_lines: Vec<Line> = app
                .diff_lines
                .iter()
                .skip(app.scroll_offset)
                .take(content_height)
                .map(|diff_line| {
                    let (prefix, style) = match diff_line.tag {
                        ChangeTag::Delete => (
                            "-",
                            Style::default()
                                .fg(Color::Red)
                                .add_modifier(Modifier::DIM),
                        ),
                        ChangeTag::Insert => (
                            "+",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::DIM),
                        ),
                        ChangeTag::Equal => (
                            " ",
                            Style::default(),
                        ),
                    };

                    Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(&diff_line.content, style),
                    ])
                })
                .collect();

            let diff_widget = Paragraph::new(visible_lines)
                .block(Block::default().borders(Borders::ALL).title("Diff"))
                .wrap(Wrap { trim: false });

            f.render_widget(diff_widget, chunks[1]);

            // Status bar
            let status_text = if let Some(ref msg) = app.status_message {
                vec![Line::from(Span::styled(
                    msg,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))]
            } else {
                vec![Line::from(vec![
                    Span::raw("Commands: "),
                    Span::styled("[q]", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Quit  "),
                    Span::styled("[s]", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Select source  "),
                    Span::styled("[t]", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Select target  "),
                    Span::styled("[c]", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Copy  "),
                    Span::styled("[e]", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Export  "),
                    Span::styled("[↑/↓]", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" Scroll"),
                ])]
            };

            let status_bar = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(status_bar, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('c') => {
                    match app.copy_to_clipboard() {
                        Ok(_) => {
                            app.status_message = Some("Diff copied to clipboard!".to_string());
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Error: {}", e));
                        }
                    }
                }
                KeyCode::Char('e') => {
                    match app.export_to_file() {
                        Ok(filename) => {
                            app.status_message = Some(format!("Diff exported to {}", filename));
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Error: {}", e));
                        }
                    }
                }
                KeyCode::Up => {
                    app.status_message = None;
                    app.scroll_up();
                }
                KeyCode::Down => {
                    app.status_message = None;
                    let content_height = terminal.size()?.height.saturating_sub(8) as usize;
                    app.scroll_down(content_height);
                }
                _ => {
                    app.status_message = None;
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match (&args.source, &args.target) {
        (Some(source), Some(target)) => {
            // Validate both files exist
            if let Err(e) = validate_file(source, "Source") {
                eprintln!("Error: {}", e);
                process::exit(1);
            }

            if let Err(e) = validate_file(target, "Target") {
                eprintln!("Error: {}", e);
                process::exit(1);
            }

            // Setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            // Create app and run
            let app = App::new(source.clone(), target.clone())?;
            let res = run_app(&mut terminal, app);

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            if let Err(err) = res {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        }
        (Some(source), None) => {
            // Validate source file exists
            if let Err(e) = validate_file(source, "Source") {
                eprintln!("Error: {}", e);
                process::exit(1);
            }

            eprintln!("Source file: {}, target file not specified", source);
            process::exit(1);
        }
        (None, Some(target)) => {
            // Validate target file exists
            if let Err(e) = validate_file(target, "Target") {
                eprintln!("Error: {}", e);
                process::exit(1);
            }

            eprintln!("Target file: {}, source file not specified", target);
            process::exit(1);
        }
        (None, None) => {
            eprintln!("No files specified. Usage: lazydiff <source> <target>");
            process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn create_test_files() -> Result<(String, String), Box<dyn std::error::Error>> {
        let source_path = "test_source_temp.txt";
        let target_path = "test_target_temp.txt";

        let mut source_file = fs::File::create(source_path)?;
        source_file.write_all(b"Line 1\nLine 2\nLine 3\nLine to remove\n")?;

        let mut target_file = fs::File::create(target_path)?;
        target_file.write_all(b"Line 1\nLine 2 modified\nLine 3\nLine added\n")?;

        Ok((source_path.to_string(), target_path.to_string()))
    }

    fn cleanup_test_files(source: &str, target: &str) {
        let _ = fs::remove_file(source);
        let _ = fs::remove_file(target);
    }

    #[test]
    fn test_generate_patch() -> Result<(), Box<dyn std::error::Error>> {
        let (source, target) = create_test_files()?;
        let app = App::new(source.clone(), target.clone())?;

        let patch = app.generate_patch();

        // Verify patch header
        assert!(patch.contains(&format!("--- {}", source)));
        assert!(patch.contains(&format!("+++ {}", target)));

        // Verify patch contains unchanged lines with space prefix
        assert!(patch.contains(" Line 1"));
        assert!(patch.contains(" Line 3"));

        // Verify patch contains removed lines with - prefix
        assert!(patch.contains("-Line 2"));
        assert!(patch.contains("-Line to remove"));

        // Verify patch contains added lines with + prefix
        assert!(patch.contains("+Line 2 modified"));
        assert!(patch.contains("+Line added"));

        cleanup_test_files(&source, &target);
        Ok(())
    }

    #[test]
    fn test_export_to_file() -> Result<(), Box<dyn std::error::Error>> {
        let (source, target) = create_test_files()?;
        let app = App::new(source.clone(), target.clone())?;

        // Export the patch
        let filename = app.export_to_file()?;

        // Verify file was created
        assert!(Path::new(&filename).exists());

        // Verify filename format
        assert!(filename.starts_with("diff_"));
        assert!(filename.ends_with(".patch"));

        // Read and verify file contents
        let contents = fs::read_to_string(&filename)?;
        assert!(!contents.is_empty(), "Patch file should not be empty");
        assert!(contents.contains(&format!("--- {}", source)));
        assert!(contents.contains(&format!("+++ {}", target)));

        // Verify patch has proper structure - should contain some content
        let line_count = contents.lines().count();
        assert!(line_count > 2, "Patch should have more than just headers");

        // Cleanup
        cleanup_test_files(&source, &target);
        fs::remove_file(&filename)?;

        Ok(())
    }

    #[test]
    fn test_export_creates_unique_filenames() -> Result<(), Box<dyn std::error::Error>> {
        let (source, target) = create_test_files()?;
        let app = App::new(source.clone(), target.clone())?;

        // Export twice
        let filename1 = app.export_to_file()?;
        assert!(Path::new(&filename1).exists(), "First export file should exist");

        std::thread::sleep(std::time::Duration::from_secs(1));
        let filename2 = app.export_to_file()?;
        assert!(Path::new(&filename2).exists(), "Second export file should exist");

        // Verify different filenames
        assert_ne!(filename1, filename2);

        // Cleanup - remove patch files first, then test files
        if Path::new(&filename1).exists() {
            fs::remove_file(&filename1)?;
        }
        if Path::new(&filename2).exists() {
            fs::remove_file(&filename2)?;
        }
        cleanup_test_files(&source, &target);

        Ok(())
    }

    #[test]
    fn test_patch_format_with_no_changes() -> Result<(), Box<dyn std::error::Error>> {
        let source_path = "test_identical_source.txt";
        let target_path = "test_identical_target.txt";

        let mut source_file = fs::File::create(source_path)?;
        source_file.write_all(b"Same content\n")?;

        let mut target_file = fs::File::create(target_path)?;
        target_file.write_all(b"Same content\n")?;

        let app = App::new(source_path.to_string(), target_path.to_string())?;
        let patch = app.generate_patch();

        // Verify header exists
        assert!(patch.starts_with("---"));
        assert!(patch.contains("+++"));

        // Verify all lines are unchanged (space prefix)
        assert!(patch.contains(" Same content"));

        // Make sure there are no actual deletions (not counting the header ---)
        let lines: Vec<&str> = patch.lines().collect();
        let has_deletions = lines.iter().skip(2).any(|line| line.starts_with('-'));
        assert!(!has_deletions);

        // Make sure there are no additions (not counting the header +++)
        let has_additions = lines.iter().skip(2).any(|line| line.starts_with('+'));
        assert!(!has_additions);

        cleanup_test_files(source_path, target_path);
        Ok(())
    }
}
