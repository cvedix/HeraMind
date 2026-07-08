//! Tests for the `message` command and its subcommands.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test message command help.
#[test]
fn test_message_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("message").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Message management commands"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("send"))
        .stdout(predicate::str::contains("read"));
}

/// Test message list help.
#[test]
fn test_message_list_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("message").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List messages"));
}

/// Test message get help.
#[test]
fn test_message_get_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("message").arg("get").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get message details"));
}

/// Test message send help.
#[test]
fn test_message_send_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("message").arg("send").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Send a new message"));
}

/// Test message read help.
#[test]
fn test_message_read_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("message").arg("read").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Acknowledge/read a message"));
}
