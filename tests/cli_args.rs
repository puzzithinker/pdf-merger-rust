use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn cli_requires_output_argument() {
    Command::cargo_bin("pdf-merger-cli")
        .expect("binary exists")
        .args(["file1.pdf"])
        .assert()
        .failure();
}
