//! Tests for the `dashboard` command and its subcommands.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test dashboard command help.
#[test]
fn test_dashboard_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("dashboard").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dashboard management commands"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("share"));
}

/// Test dashboard list help.
#[test]
fn test_dashboard_list_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("dashboard").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all dashboards"));
}

/// Test dashboard get help.
#[test]
fn test_dashboard_get_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("dashboard").arg("get").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get dashboard details"));
}

/// Test dashboard create help.
#[test]
fn test_dashboard_create_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("dashboard").arg("create").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create a new dashboard"));
}

/// Test dashboard update help.
#[test]
fn test_dashboard_update_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("dashboard").arg("update").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Update dashboard"));
}

/// Test dashboard delete help.
#[test]
fn test_dashboard_delete_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("dashboard").arg("delete").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete dashboard"));
}

/// Test dashboard share help.
#[test]
fn test_dashboard_share_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("dashboard").arg("share").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Share dashboard"));
}
