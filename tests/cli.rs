use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_no_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No files specified, entering interactive mode"));

    Ok(())
}

#[test]
fn test_help_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A terminal-based diff viewer"))
        .stdout(predicate::str::contains("Usage:"));

    Ok(())
}

#[test]
fn test_version_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("lazydiff"));

    Ok(())
}

#[test]
fn test_source_file_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.arg("nonexistent.txt");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Source file 'nonexistent.txt' does not exist"));

    Ok(())
}

#[test]
fn test_target_file_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = assert_fs::TempDir::new()?;
    let source_file = temp_dir.child("source.txt");
    source_file.write_str("test content")?;

    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.arg(source_file.path()).arg("nonexistent.txt");
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Target file 'nonexistent.txt' does not exist"));

    Ok(())
}

#[test]
fn test_source_is_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = assert_fs::TempDir::new()?;

    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.arg(temp_dir.path());
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("is not a file"));

    Ok(())
}

#[test]
fn test_both_files_exist() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = assert_fs::TempDir::new()?;
    let source_file = temp_dir.child("source.txt");
    let target_file = temp_dir.child("target.txt");

    source_file.write_str("source content")?;
    target_file.write_str("target content")?;

    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.arg(source_file.path()).arg(target_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Comparing"));

    Ok(())
}

#[test]
fn test_single_source_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = assert_fs::TempDir::new()?;
    let source_file = temp_dir.child("source.txt");
    source_file.write_str("source content")?;

    let mut cmd = Command::cargo_bin("lazydiff")?;
    cmd.arg(source_file.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Source file"))
        .stdout(predicate::str::contains("target file not specified"));

    Ok(())
}
