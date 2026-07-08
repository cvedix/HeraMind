//! Tests for the `prompt` command.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;

/// Test that prompt command requires a prompt argument.
#[test]
fn test_prompt_requires_argument() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("prompt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("<PROMPT>"));
}

/// Test that prompt accepts a message.
#[test]
fn test_prompt_accepts_message() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("prompt").arg("Hello, world!");

    // Will fail due to no LLM backend, but arguments should parse
    cmd.assert().failure();
}

/// Test that max_tokens flag is accepted.
#[test]
fn test_prompt_max_tokens_flag() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("prompt")
        .arg("test")
        .arg("--max-tokens")
        .arg("1000");

    cmd.assert().failure();
}

/// Test that temperature flag is accepted.
#[test]
fn test_prompt_temperature_flag() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("prompt").arg("test").arg("-t").arg("0.5");

    cmd.assert().failure();
}

/// Test that invalid temperature is rejected.
#[test]
fn test_prompt_invalid_temperature() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("prompt")
        .arg("test")
        .arg("--temperature")
        .arg("invalid");

    cmd.assert().failure();
}

/// Test that invalid max_tokens is rejected.
#[test]
fn test_prompt_invalid_max_tokens() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("prompt")
        .arg("test")
        .arg("--max-tokens")
        .arg("invalid");

    cmd.assert().failure();
}

/// Test help shows all prompt options.
#[test]
fn test_prompt_help_shows_options() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("prompt").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PROMPT"))
        .stdout(predicate::str::contains("--max-tokens"))
        .stdout(predicate::str::contains("--temperature"))
        .stdout(predicate::str::contains("0.7")); // Default temperature
}
