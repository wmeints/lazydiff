use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

impl FileBrowser {
    pub fn new() -> Result<Self, io::Error> {
        let current_dir = env::current_dir()?;
        let mut browser = FileBrowser {
            current_dir: current_dir.clone(),
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
        };
        browser.load_entries()?;
        Ok(browser)
    }

    pub fn load_entries(&mut self) -> Result<(), io::Error> {
        self.entries.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;

        // Add parent directory entry if not at root
        if self.current_dir.parent().is_some() {
            self.entries.push(PathBuf::from(".."));
        }

        // Read directory entries
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.current_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect();

        // Sort: directories first, then files, alphabetically
        entries.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        self.entries.extend(entries);
        Ok(())
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.entries.len() {
            self.selected_index += 1;
        }
    }

    pub fn update_scroll(&mut self, viewport_height: usize) {
        if self.selected_index >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.selected_index - viewport_height + 1;
        } else if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    pub fn enter_selected(&mut self) -> Result<Option<PathBuf>, io::Error> {
        if self.entries.is_empty() {
            return Ok(None);
        }

        let selected = &self.entries[self.selected_index];

        // Handle parent directory
        if selected.to_str() == Some("..") {
            if let Some(parent) = self.current_dir.parent() {
                self.current_dir = parent.to_path_buf();
                self.load_entries()?;
            }
            return Ok(None);
        }

        let full_path = if selected.is_absolute() {
            selected.clone()
        } else {
            self.current_dir.join(selected)
        };

        if full_path.is_dir() {
            self.current_dir = full_path;
            self.load_entries()?;
            Ok(None)
        } else if full_path.is_file() {
            Ok(Some(full_path))
        } else {
            Ok(None)
        }
    }

    pub fn get_display_name(&self, path: &PathBuf) -> String {
        if path.to_str() == Some("..") {
            return "..".to_string();
        }

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();

        if path.is_absolute() && path.is_dir() {
            format!("{}/", name)
        } else if !path.is_absolute() {
            name
        } else if self.current_dir.join(path).is_dir() {
            format!("{}/", name)
        } else {
            name
        }
    }
}
