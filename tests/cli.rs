use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_download_file() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("10MB.bin");

    let mut cmd = Command::cargo_bin("dw").unwrap();
    cmd.arg("http://cachefly.cachefly.net/10mb.test")
        .arg("--output")
        .arg(output_path.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Download saved as:"));

    assert!(output_path.exists());
    assert!(output_path.metadata().unwrap().len() > 0);
}

#[test]
fn test_parallel_download_file() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("100MB.bin");

    let mut cmd = Command::cargo_bin("dw").unwrap();
    cmd.arg("http://cachefly.cachefly.net/100mb.test")
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .arg("-c")
        .arg("4");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Download saved as:"));

    assert!(output_path.exists());
    let metadata = output_path.metadata().unwrap();
    assert_eq!(metadata.len(), 104857600);
}

#[test]
fn test_failed_download() {
    let mut cmd = Command::cargo_bin("dw").unwrap();
    cmd.arg("http://cachefly.cachefly.net/nonexistent-file.bin");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Download failed"));
}

#[test]
fn test_help_message() {
    let mut cmd = Command::cargo_bin("dw").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "A blazingly fast download accelerator, written in Rust.",
    ));
}
