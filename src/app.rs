use crate::browser::FileBrowser;
use crate::diff::{self, DiffLine};
use crate::ui;
use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use std::fs;
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    DiffView,
    SelectingSource,
    SelectingTarget,
    SelectionMode,
}

pub struct App {
    pub source_file: String,
    pub target_file: String,
    pub diff_lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub cursor_position: usize,
    pub status_message: Option<String>,
    pub clipboard: Option<Clipboard>,
    pub mode: AppMode,
    pub file_browser: FileBrowser,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
}

impl App {
    pub fn new(
        source_file: String,
        target_file: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let source_content = fs::read_to_string(&source_file)?;
        let target_content = fs::read_to_string(&target_file)?;

        let diff_lines = diff::generate_diff(&source_content, &target_content);

        // Try to initialize clipboard, but allow it to fail gracefully
        let clipboard = Clipboard::new().ok();
        let file_browser = FileBrowser::new()?;

        Ok(App {
            source_file,
            target_file,
            diff_lines,
            scroll_offset: 0,
            cursor_position: 0,
            status_message: None,
            clipboard,
            mode: AppMode::DiffView,
            file_browser,
            selection_start: None,
            selection_end: None,
        })
    }

    pub fn new_empty(initial_mode: AppMode) -> Result<Self, Box<dyn std::error::Error>> {
        // Try to initialize clipboard, but allow it to fail gracefully
        let clipboard = Clipboard::new().ok();
        let file_browser = FileBrowser::new()?;

        Ok(App {
            source_file: String::new(),
            target_file: String::new(),
            diff_lines: Vec::new(),
            scroll_offset: 0,
            cursor_position: 0,
            status_message: Some("Please select a file".to_string()),
            clipboard,
            mode: initial_mode,
            file_browser,
            selection_start: None,
            selection_end: None,
        })
    }

    pub fn regenerate_diff(&mut self) -> Result<(), io::Error> {
        let source_content = fs::read_to_string(&self.source_file)?;
        let target_content = fs::read_to_string(&self.target_file)?;

        self.diff_lines = diff::generate_diff(&source_content, &target_content);
        self.scroll_offset = 0;
        Ok(())
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, max_visible_lines: usize) {
        if self.scroll_offset + max_visible_lines < self.diff_lines.len() {
            self.scroll_offset += 1;
        }
    }

    fn generate_patch(&self) -> String {
        let line_range = self.get_selection_range();
        diff::generate_patch(
            &self.source_file,
            &self.target_file,
            &self.diff_lines,
            line_range,
        )
    }

    pub fn copy_to_clipboard(&mut self) -> Result<(), String> {
        let patch = self.generate_patch();
        match &mut self.clipboard {
            Some(clipboard) => diff::copy_to_clipboard(clipboard, &patch),
            None => Err("Clipboard not available in this environment".to_string()),
        }
    }

    pub fn export_to_file(&self) -> Result<String, String> {
        let patch = self.generate_patch();
        diff::export_to_file(&patch)
    }

    pub fn enter_selection_mode(&mut self) {
        self.mode = AppMode::SelectionMode;
        self.cursor_position = self.scroll_offset;
        self.selection_start = None;
        self.selection_end = None;
        self.status_message =
            Some("SELECTION MODE - Press Space to mark start/end, v to exit".to_string());
    }

    pub fn exit_selection_mode(&mut self) {
        self.mode = AppMode::DiffView;
        self.selection_start = None;
        self.selection_end = None;
        self.status_message = Some("Selection mode exited".to_string());
    }

    pub fn toggle_selection_anchor(&mut self) {
        if self.selection_start.is_none() {
            // Set the start of selection at current cursor position
            self.selection_start = Some(self.cursor_position);
            self.selection_end = Some(self.cursor_position);
            self.status_message = Some(format!("Selection start: line {}", self.cursor_position));
        } else {
            // Finalize the selection
            if let Some(start) = self.selection_start {
                self.status_message = Some(format!(
                    "Selection: lines {}-{} ({} lines selected)",
                    start.min(self.cursor_position),
                    start.max(self.cursor_position),
                    (start as i32 - self.cursor_position as i32).abs() + 1
                ));
            }
        }
    }

    pub fn update_selection_end(&mut self) {
        if self.selection_start.is_some() {
            self.selection_end = Some(self.cursor_position);
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            // Scroll up if cursor moves above visible area
            if self.cursor_position < self.scroll_offset {
                self.scroll_offset = self.cursor_position;
            }
        }
    }

    pub fn cursor_down(&mut self, max_visible_lines: usize) {
        if self.cursor_position + 1 < self.diff_lines.len() {
            self.cursor_position += 1;
            // Scroll down if cursor moves below visible area
            if self.cursor_position >= self.scroll_offset + max_visible_lines {
                self.scroll_offset = self.cursor_position - max_visible_lines + 1;
            }
        }
    }

    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        match (self.selection_start, self.selection_end) {
            (Some(start), Some(end)) => {
                let min = start.min(end);
                let max = start.max(end);
                Some((min, max))
            }
            _ => None,
        }
    }
}

fn handle_file_selection(app: &mut App) {
    match app.file_browser.enter_selected() {
        Ok(Some(selected_file)) => {
            // File was selected
            if let Some(file_path) = selected_file.to_str() {
                if app.mode == AppMode::SelectingSource {
                    app.source_file = file_path.to_string();

                    // If target is not set, move to selecting target
                    if app.target_file.is_empty() {
                        app.mode = AppMode::SelectingTarget;
                        app.status_message =
                            Some(format!("Source: {} - Now select target file", file_path));
                        let _ = app.file_browser.load_entries();
                    } else {
                        // Both files are set, regenerate diff
                        if let Err(e) = app.regenerate_diff() {
                            app.status_message = Some(format!("Error loading files: {}", e));
                        } else {
                            app.status_message =
                                Some(format!("Source file updated: {}", file_path));
                        }
                        app.mode = AppMode::DiffView;
                    }
                } else {
                    app.target_file = file_path.to_string();

                    // If source is not set, move to selecting source
                    if app.source_file.is_empty() {
                        app.mode = AppMode::SelectingSource;
                        app.status_message =
                            Some(format!("Target: {} - Now select source file", file_path));
                        let _ = app.file_browser.load_entries();
                    } else {
                        // Both files are set, regenerate diff
                        if let Err(e) = app.regenerate_diff() {
                            app.status_message = Some(format!("Error loading files: {}", e));
                        } else {
                            app.status_message =
                                Some(format!("Target file updated: {}", file_path));
                        }
                        app.mode = AppMode::DiffView;
                    }
                }
            }
        }
        Ok(None) => {
            // Directory was entered, nothing to do
        }
        Err(e) => {
            app.status_message = Some(format!("Error: {}", e));
        }
    }
}

fn handle_browser_input<B: ratatui::backend::Backend>(
    app: &mut App,
    key_code: KeyCode,
    terminal: &Terminal<B>,
) -> io::Result<bool> {
    let content_height = terminal.size()?.height.saturating_sub(8) as usize;

    match key_code {
        KeyCode::Up => {
            app.file_browser.move_up();
        }
        KeyCode::Down => {
            app.file_browser.move_down();
            app.file_browser.update_scroll(content_height);
        }
        KeyCode::Enter => {
            handle_file_selection(app);
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            // Only return to diff view if both files are set
            if !app.source_file.is_empty() && !app.target_file.is_empty() {
                app.mode = AppMode::DiffView;
            } else {
                // Exit the application if files aren't set
                return Ok(true);
            }
        }
        _ => {}
    }

    Ok(false)
}

fn handle_diffview_input<B: ratatui::backend::Backend>(
    app: &mut App,
    key_code: KeyCode,
    terminal: &Terminal<B>,
) -> io::Result<bool> {
    match key_code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('s') => {
            app.mode = AppMode::SelectingSource;
            let _ = app.file_browser.load_entries();
        }
        KeyCode::Char('t') => {
            app.mode = AppMode::SelectingTarget;
            let _ = app.file_browser.load_entries();
        }
        KeyCode::Char('v') => {
            app.enter_selection_mode();
        }
        KeyCode::Char('c') => match app.copy_to_clipboard() {
            Ok(_) => {
                app.status_message = Some("Diff copied to clipboard!".to_string());
            }
            Err(e) => {
                app.status_message = Some(format!("Error: {}", e));
            }
        },
        KeyCode::Char('e') => match app.export_to_file() {
            Ok(filename) => {
                app.status_message = Some(format!("Diff exported to {}", filename));
            }
            Err(e) => {
                app.status_message = Some(format!("Error: {}", e));
            }
        },
        KeyCode::Up => {
            app.scroll_up();
        }
        KeyCode::Down => {
            let content_height = terminal.size()?.height.saturating_sub(8) as usize;
            app.scroll_down(content_height);
        }
        _ => {}
    }

    Ok(false)
}

fn handle_selection_input<B: ratatui::backend::Backend>(
    app: &mut App,
    key_code: KeyCode,
    terminal: &Terminal<B>,
) -> io::Result<bool> {
    match key_code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('v') => {
            app.exit_selection_mode();
        }
        KeyCode::Char(' ') => {
            app.toggle_selection_anchor();
        }
        KeyCode::Char('c') => {
            if app.get_selection_range().is_some() {
                match app.copy_to_clipboard() {
                    Ok(_) => {
                        app.status_message = Some("Selection copied to clipboard!".to_string());
                    }
                    Err(e) => {
                        app.status_message = Some(format!("Error: {}", e));
                    }
                }
            } else {
                app.status_message =
                    Some("No selection made. Press Space to mark start/end.".to_string());
            }
        }
        KeyCode::Char('e') => {
            if app.get_selection_range().is_some() {
                match app.export_to_file() {
                    Ok(filename) => {
                        app.status_message = Some(format!("Selection exported to {}", filename));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("Error: {}", e));
                    }
                }
            } else {
                app.status_message =
                    Some("No selection made. Press Space to mark start/end.".to_string());
            }
        }
        KeyCode::Up => {
            app.cursor_up();
            app.update_selection_end();
        }
        KeyCode::Down => {
            let content_height = terminal.size()?.height.saturating_sub(8) as usize;
            app.cursor_down(content_height);
            app.update_selection_end();
        }
        _ => {}
    }

    Ok(false)
}

pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            ui::render_ui(f, &app);
        })?;

        if let Event::Key(key) = event::read()? {
            app.status_message = None;

            let should_exit = match app.mode {
                AppMode::DiffView => handle_diffview_input(&mut app, key.code, terminal)?,
                AppMode::SelectingSource | AppMode::SelectingTarget => {
                    handle_browser_input(&mut app, key.code, terminal)?
                }
                AppMode::SelectionMode => handle_selection_input(&mut app, key.code, terminal)?,
            };

            if should_exit {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::Mutex;

    // Mutex to serialize clipboard access during tests
    static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

    fn create_test_files() -> Result<(String, String), Box<dyn std::error::Error>> {
        use std::thread;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let thread_id = format!("{:?}", thread::current().id());
        let unique_suffix = format!(
            "{}_{}",
            timestamp,
            thread_id.replace("ThreadId(", "").replace(")", "")
        );

        let source_path = format!("test_source_{}_.txt", unique_suffix);
        let target_path = format!("test_target_{}_.txt", unique_suffix);

        let mut source_file = fs::File::create(&source_path)?;
        source_file.write_all(b"Line 1\nLine 2\nLine 3\nLine to remove\n")?;

        let mut target_file = fs::File::create(&target_path)?;
        target_file.write_all(b"Line 1\nLine 2 modified\nLine 3\nLine added\n")?;

        Ok((source_path, target_path))
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

        assert!(patch.contains(&format!("--- {}", source)));
        assert!(patch.contains(&format!("+++ {}", target)));
        assert!(patch.contains(" Line 1"));
        assert!(patch.contains(" Line 3"));
        assert!(patch.contains("-Line 2"));
        assert!(patch.contains("-Line to remove"));
        assert!(patch.contains("+Line 2 modified"));
        assert!(patch.contains("+Line added"));

        cleanup_test_files(&source, &target);
        Ok(())
    }

    #[test]
    fn test_export_to_file() -> Result<(), Box<dyn std::error::Error>> {
        let (source, target) = create_test_files()?;
        let app = App::new(source.clone(), target.clone())?;

        let filename = app.export_to_file()?;

        assert!(std::path::Path::new(&filename).exists());
        assert!(filename.starts_with("diff_"));
        assert!(filename.ends_with(".patch"));

        let contents = fs::read_to_string(&filename)?;
        assert!(!contents.is_empty());
        assert!(contents.contains(&format!("--- {}", source)));
        assert!(contents.contains(&format!("+++ {}", target)));

        let line_count = contents.lines().count();
        assert!(line_count > 2);

        cleanup_test_files(&source, &target);
        fs::remove_file(&filename)?;

        Ok(())
    }

    #[test]
    fn test_export_creates_unique_filenames() -> Result<(), Box<dyn std::error::Error>> {
        let (source, target) = create_test_files()?;
        let app = App::new(source.clone(), target.clone())?;

        let filename1 = app.export_to_file()?;
        assert!(std::path::Path::new(&filename1).exists());

        std::thread::sleep(std::time::Duration::from_secs(1));
        let filename2 = app.export_to_file()?;
        assert!(std::path::Path::new(&filename2).exists());

        assert_ne!(filename1, filename2);

        if std::path::Path::new(&filename1).exists() {
            fs::remove_file(&filename1)?;
        }
        if std::path::Path::new(&filename2).exists() {
            fs::remove_file(&filename2)?;
        }
        cleanup_test_files(&source, &target);

        Ok(())
    }

    #[test]
    fn test_patch_format_with_no_changes() -> Result<(), Box<dyn std::error::Error>> {
        use std::thread;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let thread_id = format!("{:?}", thread::current().id());
        let unique_suffix = format!(
            "{}_{}",
            timestamp,
            thread_id.replace("ThreadId(", "").replace(")", "")
        );

        let source_path = format!("test_identical_source_{}_.txt", unique_suffix);
        let target_path = format!("test_identical_target_{}_.txt", unique_suffix);

        let mut source_file = fs::File::create(&source_path)?;
        source_file.write_all(b"Same content\n")?;

        let mut target_file = fs::File::create(&target_path)?;
        target_file.write_all(b"Same content\n")?;

        let app = App::new(source_path.to_string(), target_path.to_string())?;
        let patch = app.generate_patch();

        assert!(patch.starts_with("---"));
        assert!(patch.contains("+++"));
        assert!(patch.contains(" Same content"));

        let lines: Vec<&str> = patch.lines().collect();
        let has_deletions = lines.iter().skip(2).any(|line| line.starts_with('-'));
        assert!(!has_deletions);

        let has_additions = lines.iter().skip(2).any(|line| line.starts_with('+'));
        assert!(!has_additions);

        cleanup_test_files(&source_path, &target_path);
        Ok(())
    }

    #[test]
    fn test_copy_to_clipboard() -> Result<(), Box<dyn std::error::Error>> {
        let _lock = CLIPBOARD_LOCK.lock().unwrap();

        let (source, target) = create_test_files()?;
        let mut app = App::new(source.clone(), target.clone())?;

        let result = app.copy_to_clipboard();

        match result {
            Ok(_) => {
                if let Some(clipboard) = &mut app.clipboard {
                    let clipboard_content = clipboard
                        .get_text()
                        .expect("Should read clipboard after successful copy");

                    assert!(clipboard_content.contains(&format!("--- {}", source)));
                    assert!(clipboard_content.contains(&format!("+++ {}", target)));
                    assert!(!clipboard_content.is_empty());
                }
            }
            Err(e) => {
                eprintln!("Clipboard not available: {}", e);
            }
        }

        cleanup_test_files(&source, &target);
        Ok(())
    }

    #[test]
    fn test_clipboard_contains_correct_patch() -> Result<(), Box<dyn std::error::Error>> {
        let _lock = CLIPBOARD_LOCK.lock().unwrap();

        let (source, target) = create_test_files()?;
        let mut app = App::new(source.clone(), target.clone())?;

        let expected_patch = app.generate_patch();

        if let Ok(_) = app.copy_to_clipboard() {
            if let Some(clipboard) = &mut app.clipboard {
                if let Ok(clipboard_content) = clipboard.get_text() {
                    assert!(
                        clipboard_content.contains(&format!("--- {}", source)),
                        "Clipboard should contain source file header"
                    );
                    assert!(
                        clipboard_content.contains(&format!("+++ {}", target)),
                        "Clipboard should contain target file header"
                    );
                    assert!(
                        !clipboard_content.is_empty(),
                        "Clipboard should not be empty"
                    );
                    assert!(
                        clipboard_content.lines().count() > 2,
                        "Clipboard should have more than just headers"
                    );
                    assert_eq!(
                        clipboard_content, expected_patch,
                        "Clipboard content should exactly match generated patch"
                    );
                }
            }
        }

        cleanup_test_files(&source, &target);
        Ok(())
    }

    #[test]
    fn test_multiple_clipboard_copies() -> Result<(), Box<dyn std::error::Error>> {
        let _lock = CLIPBOARD_LOCK.lock().unwrap();

        use std::thread;
        use std::time::{SystemTime, UNIX_EPOCH};

        // First copy
        let (source1, target1) = create_test_files()?;
        let mut app1 = App::new(source1.clone(), target1.clone())?;
        let patch1 = app1.generate_patch();

        if let Ok(_) = app1.copy_to_clipboard() {
            if let Some(clipboard) = &mut app1.clipboard {
                if let Ok(content) = clipboard.get_text() {
                    assert_eq!(content, patch1);
                }
            }
        }

        cleanup_test_files(&source1, &target1);

        // Second copy with different content
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let thread_id = format!("{:?}", thread::current().id());
        let unique_suffix = format!(
            "{}_{}",
            timestamp,
            thread_id.replace("ThreadId(", "").replace(")", "")
        );

        let source2_path = format!("test_source2_{}_.txt", unique_suffix);
        let target2_path = format!("test_target2_{}_.txt", unique_suffix);

        let mut source_file = fs::File::create(&source2_path)?;
        source_file.write_all(b"Different line 1\nDifferent line 2\n")?;

        let mut target_file = fs::File::create(&target2_path)?;
        target_file.write_all(b"Different line 1\nModified line 2\n")?;

        let mut app2 = App::new(source2_path.to_string(), target2_path.to_string())?;
        let patch2 = app2.generate_patch();

        if let Ok(_) = app2.copy_to_clipboard() {
            if let Some(clipboard) = &mut app2.clipboard {
                if let Ok(content) = clipboard.get_text() {
                    assert_eq!(content, patch2);
                    assert_ne!(content, patch1, "Second copy should overwrite first");
                }
            }
        }

        cleanup_test_files(&source2_path, &target2_path);
        Ok(())
    }
}
