//! Tests for the `device` command and its subcommands.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test device command help.
#[test]
fn test_device_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Device management commands"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("history"))
        .stdout(predicate::str::contains("control"))
        .stdout(predicate::str::contains("types"))
        .stdout(predicate::str::contains("webhook-url"))
        .stdout(predicate::str::contains("drafts"));
}

/// Test device list help.
#[test]
fn test_device_list_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all devices"));
}

/// Test device get help.
#[test]
fn test_device_get_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("get").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get device details"));
}

/// Test device create help.
#[test]
fn test_device_create_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("create").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create a new device"));
}

/// Test device update help.
#[test]
fn test_device_update_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("update").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Update device"));
}

/// Test device delete help.
#[test]
fn test_device_delete_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("delete").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete device"));
}

/// Test device history help.
#[test]
fn test_device_history_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("history").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get telemetry history"));
}

/// Test device control help.
#[test]
fn test_device_control_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("control").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Send control command"));
}

/// Test device types help.
#[test]
fn test_device_types_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("device").arg("types").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Device type management"));
}
