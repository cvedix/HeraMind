//! Basic CLI tests for heramind command-line interface.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;

// Include command-specific test modules
mod commands;

/// Test that the CLI binary exists and shows help.
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("HeraMind AI Agent"))
        .stdout(predicate::str::contains("Run LLMs on edge devices"))
        .stdout(predicate::str::contains("serve"))
        .stdout(predicate::str::contains("prompt"))
        .stdout(predicate::str::contains("chat"))
        .stdout(predicate::str::contains("list-models"))
        .stdout(predicate::str::contains("extension"));
}

/// Test that the CLI shows version information.
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("heramind"));
}

/// Test the verbose flag is accepted (even if it fails due to missing LLM).
#[test]
fn test_verbose_flag_accepted() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("--verbose").arg("--help");

    cmd.assert().success();
}

/// Test that the model flag is accepted as a global flag.
#[test]
fn test_model_flag_accepted() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("--model").arg("qwen3-vl:2b").arg("--help");

    cmd.assert().success();
}

/// Test that providing no subcommand shows an error.
#[test]
fn test_no_subcommand_shows_error() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();

    // Clap displays usage on stdout with exit code 2
    cmd.assert().failure().code(2); // Clap's error code for missing required subcommand
}
