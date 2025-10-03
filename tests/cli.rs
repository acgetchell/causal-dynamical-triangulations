//! Command-line interface integration tests for the CDT-RS application.
//!
//! This module contains tests that verify the behavior of the command-line
//! interface, including argument validation, success scenarios, and error handling.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn exit_success() {
    let mut cmd = Command::cargo_bin("cdt-rs").unwrap();
    cmd.arg("-v");
    cmd.arg("32");
    cmd.arg("-t");
    cmd.arg("3");
    cmd.assert().success();
}

#[test]
fn cdt_cli_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("-v");
    cmd.arg("32");
    cmd.arg("-t");
    cmd.arg("3");
    cmd.env("RUST_LOG", "info");

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Triangles"));

    Ok(())
}

#[test]
fn cdt_cli_no_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.assert().failure().stderr(predicate::str::contains(
        "error: the following required arguments were not provided:",
    ));

    Ok(())
}

#[test]
fn cdt_cli_invalid_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("-v");
    cmd.arg("32");
    cmd.arg("-t");
    cmd.arg("3");
    cmd.arg("-d");
    cmd.arg("5");

    cmd.assert().failure().stderr(predicate::str::contains(
        "error: invalid value '5' for '--dimension <DIMENSION>': 5 is not in 2..4",
    ));

    Ok(())
}

#[test]
fn cdt_cli_out_of_range_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("-v");
    cmd.arg("32");
    cmd.arg("-t");
    cmd.arg("3");
    cmd.arg("-d");
    cmd.arg("3");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported dimension: 3"));

    Ok(())
}
