//! Tests for the `chat` command.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;

/// Test that chat command can be invoked.
#[test]
fn test_chat_command_exists() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("chat").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Chat mode"))
        .stdout(predicate::str::contains("interactive REPL"));
}

/// Test that session flag is accepted.
#[test]
fn test_chat_session_flag() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("chat").arg("--session").arg("test-session-123");

    // Will fail due to no LLM backend, but arguments should parse
    cmd.assert().failure();
}

/// Test that short session flag (-s) is accepted.
#[test]
fn test_chat_short_session_flag() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("chat").arg("-s").arg("my-session");

    cmd.assert().failure();
}

/// Test that chat can start without a session (creates new).
#[test]
fn test_chat_without_session() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("chat");

    // Will fail due to no LLM backend
    cmd.assert().failure();
}

/// Test help shows chat session option.
#[test]
fn test_chat_help_shows_session_option() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("chat").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--session"))
        .stdout(predicate::str::contains("Session ID"));
}
