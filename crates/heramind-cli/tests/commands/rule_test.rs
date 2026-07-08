//! Tests for the `rule` command and its subcommands.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test rule command help.
#[test]
fn test_rule_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Rule management commands"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("enable"))
        .stdout(predicate::str::contains("disable"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("history"));
}

/// Test rule list help.
#[test]
fn test_rule_list_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all rules"));
}

/// Test rule get help.
#[test]
fn test_rule_get_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("get").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get rule details"));
}

/// Test rule create help.
#[test]
fn test_rule_create_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("create").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create a new rule"));
}

/// Test rule update help.
#[test]
fn test_rule_update_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("update").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Update rule"));
}

/// Test rule delete help.
#[test]
fn test_rule_delete_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("delete").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete rule"));
}

/// Test rule enable help.
#[test]
fn test_rule_enable_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("enable").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Enable rule"));
}

/// Test rule disable help.
#[test]
fn test_rule_disable_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("disable").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Disable rule"));
}

/// Test rule test help.
#[test]
fn test_rule_test_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("test").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test rule"));
}

/// Test rule history help.
#[test]
fn test_rule_history_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("rule").arg("history").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get rule execution history"));
}
