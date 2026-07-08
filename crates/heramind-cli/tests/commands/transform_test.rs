//! Tests for the `transform` command and its subcommands.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test transform command help.
#[test]
fn test_transform_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("transform").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Transform management commands"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("metrics"))
        .stdout(predicate::str::contains("test-code"))
        .stdout(predicate::str::contains("data-sources"));
}

/// Test transform list help.
#[test]
fn test_transform_list_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("transform").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all transforms"));
}

/// Test transform metrics help.
#[test]
fn test_transform_metrics_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("transform").arg("metrics").arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "List virtual metrics from transforms",
    ));
}

/// Test transform test-code help.
#[test]
fn test_transform_test_code_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("transform").arg("test-code").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test transform code"));
}

/// Test transform data-sources help.
#[test]
fn test_transform_data_sources_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("transform").arg("data-sources").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List transform data sources"));
}
