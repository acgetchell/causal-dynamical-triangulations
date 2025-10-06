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
        .stderr(predicate::str::contains("faces"));

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

#[test]
fn cdt_cli_invalid_measurement_frequency_zero() -> Result<(), Box<dyn std::error::Error>> {
    // Note: This would be caught by clap's range validation now,
    // but we test the error message for completeness
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("--vertices").arg("10");
    cmd.arg("--timeslices").arg("3");
    cmd.arg("--measurement-frequency").arg("0");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("0 is not in 1.."));

    Ok(())
}

#[test]
fn cdt_cli_invalid_measurement_frequency_too_large() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("--vertices").arg("10");
    cmd.arg("--timeslices").arg("3");
    cmd.arg("--steps").arg("100");
    cmd.arg("--measurement-frequency").arg("200");
    cmd.arg("--simulate");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Measurement frequency cannot be greater than total steps",
    ));

    Ok(())
}

#[test]
fn cdt_cli_invalid_vertices_too_few() -> Result<(), Box<dyn std::error::Error>> {
    // This should be caught by clap's range validation
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("--vertices").arg("2");
    cmd.arg("--timeslices").arg("3");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("2 is not in 3.."));

    Ok(())
}

#[test]
fn cdt_cli_invalid_timeslices_zero() -> Result<(), Box<dyn std::error::Error>> {
    // This should be caught by clap's range validation
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("--vertices").arg("10");
    cmd.arg("--timeslices").arg("0");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("0 is not in 1.."));

    Ok(())
}

#[test]
fn cdt_cli_config_validation_comprehensive() -> Result<(), Box<dyn std::error::Error>> {
    // Test a complex scenario with valid parameters to ensure our validation doesn't break normal usage
    let mut cmd = Command::cargo_bin("cdt-rs")?;

    cmd.arg("--vertices").arg("10");
    cmd.arg("--timeslices").arg("3");
    cmd.arg("--steps").arg("50");
    cmd.arg("--measurement-frequency").arg("5");
    cmd.arg("--temperature").arg("1.5");
    cmd.arg("--thermalization-steps").arg("10");
    cmd.env("RUST_LOG", "error"); // Reduce log noise

    cmd.assert().success();

    Ok(())
}
