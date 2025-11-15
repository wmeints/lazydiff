use arboard::Clipboard;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct DiffLine {
    pub tag: ChangeTag,
    pub content: String,
}

pub fn generate_diff(source_content: &str, target_content: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(source_content, target_content);
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

    diff_lines
}

pub fn generate_patch(
    source_file: &str,
    target_file: &str,
    diff_lines: &[DiffLine],
    line_range: Option<(usize, usize)>,
) -> String {
    let mut patch = String::new();

    // Add patch header
    patch.push_str(&format!("--- {}\n", source_file));
    patch.push_str(&format!("+++ {}\n", target_file));

    // Determine which lines to include
    let lines_to_include: Vec<&DiffLine> = match line_range {
        Some((start, end)) => {
            // Filter diff_lines to only include the selected range
            diff_lines
                .iter()
                .enumerate()
                .filter(|(i, _)| *i >= start && *i <= end)
                .map(|(_, line)| line)
                .collect()
        }
        None => diff_lines.iter().collect(),
    };

    // Add diff lines in unified format
    for diff_line in lines_to_include {
        let prefix = match diff_line.tag {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        patch.push_str(&format!("{}{}\n", prefix, diff_line.content));
    }

    patch
}

pub fn copy_to_clipboard(clipboard: &mut Clipboard, patch: &str) -> Result<(), String> {
    clipboard
        .set_text(patch)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}

pub fn export_to_file(patch: &str) -> Result<String, String> {
    // Generate filename with high-precision timestamp to avoid collisions
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_nanos();

    let filename = format!("diff_{}.patch", timestamp);

    // Write patch to file (fs::write handles flushing automatically)
    fs::write(&filename, patch.as_bytes())
        .map_err(|e| format!("Failed to write to file: {}", e))?;

    Ok(filename)
}

pub fn validate_file(path: &str, file_type: &str) -> Result<(), String> {
    let file_path = Path::new(path);

    if !file_path.exists() {
        return Err(format!("{} file '{}' does not exist", file_type, path));
    }

    if !file_path.is_file() {
        return Err(format!("{} path '{}' is not a file", file_type, path));
    }

    Ok(())
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
        let source_content = fs::read_to_string(&source)?;
        let target_content = fs::read_to_string(&target)?;

        let diff_lines = generate_diff(&source_content, &target_content);
        let patch = generate_patch(&source, &target, &diff_lines, None);

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
        let source_content = fs::read_to_string(&source)?;
        let target_content = fs::read_to_string(&target)?;

        let diff_lines = generate_diff(&source_content, &target_content);
        let patch = generate_patch(&source, &target, &diff_lines, None);

        // Export the patch
        let filename = export_to_file(&patch)?;

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

        // Verify patch has proper structure
        let line_count = contents.lines().count();
        assert!(line_count > 2, "Patch should have more than just headers");

        // Cleanup
        cleanup_test_files(&source, &target);
        fs::remove_file(&filename)?;

        Ok(())
    }

    #[test]
    fn test_copy_to_clipboard() -> Result<(), Box<dyn std::error::Error>> {
        let _lock = CLIPBOARD_LOCK.lock().unwrap();

        let (source, target) = create_test_files()?;
        let source_content = fs::read_to_string(&source)?;
        let target_content = fs::read_to_string(&target)?;

        let diff_lines = generate_diff(&source_content, &target_content);
        let patch = generate_patch(&source, &target, &diff_lines, None);

        // Try to initialize clipboard, but handle gracefully if not available
        match Clipboard::new() {
            Ok(mut clipboard) => {
                let result = copy_to_clipboard(&mut clipboard, &patch);

                match result {
                    Ok(_) => {
                        let clipboard_content =
                            clipboard.get_text().expect("Should read clipboard");
                        assert!(clipboard_content.contains(&format!("--- {}", source)));
                        assert!(clipboard_content.contains(&format!("+++ {}", target)));
                        assert!(!clipboard_content.is_empty());
                    }
                    Err(e) => {
                        eprintln!("Clipboard operation failed: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Clipboard not available in this environment: {}", e);
            }
        }

        cleanup_test_files(&source, &target);
        Ok(())
    }
}
