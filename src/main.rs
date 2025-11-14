use clap::Parser;
use std::path::Path;
use std::process;

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

fn main() {
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

            println!("Comparing {} with {}", source, target);
        }
        (Some(source), None) => {
            // Validate source file exists
            if let Err(e) = validate_file(source, "Source") {
                eprintln!("Error: {}", e);
                process::exit(1);
            }

            println!("Source file: {}, target file not specified", source);
        }
        (None, Some(target)) => {
            // Validate target file exists
            if let Err(e) = validate_file(target, "Target") {
                eprintln!("Error: {}", e);
                process::exit(1);
            }

            println!("Target file: {}, source file not specified", target);
        }
        (None, None) => {
            println!("No files specified, entering interactive mode");
        }
    }
}
