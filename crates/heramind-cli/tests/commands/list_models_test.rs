//! Tests for the `list-models` command.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;

/// Test that list-models command can be invoked.
#[test]
fn test_list_models_command_exists() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("list-models").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Ollama"));
}

/// Test that list-models accepts custom endpoint.
#[test]
fn test_list_models_custom_endpoint() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("list-models")
        .arg("--endpoint")
        .arg("http://localhost:9999"); // Use unreachable port

    // The command fails when Ollama is not running
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to connect to Ollama"));
}

/// Test that list-models default endpoint is shown in help.
#[test]
fn test_list_models_default_endpoint_in_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("list-models").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("http://localhost:11434"));
}

/// Test that list-models with unreachable endpoint shows appropriate message.
#[test]
fn test_list_models_unreachable_endpoint() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("list-models")
        .arg("--endpoint")
        .arg("http://localhost:9999"); // Unlikely to be running

    // The command fails with connection error
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to connect to Ollama"));
}
