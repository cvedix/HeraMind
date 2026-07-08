//! Tests for the `agent` command and its subcommands.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test agent command help.
#[test]
fn test_agent_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Agent management commands"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("control"))
        .stdout(predicate::str::contains("invoke"))
        .stdout(predicate::str::contains("memory"))
        .stdout(predicate::str::contains("executions"));
}

/// Test agent list help.
#[test]
fn test_agent_list_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all agents"));
}

/// Test agent get help.
#[test]
fn test_agent_get_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("get").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get agent details"));
}

/// Test agent create help.
#[test]
fn test_agent_create_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("create").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create a new agent"));
}

/// Test agent update help.
#[test]
fn test_agent_update_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("update").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Update agent"));
}

/// Test agent delete help.
#[test]
fn test_agent_delete_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("delete").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete agent"));
}

/// Test agent control help.
#[test]
fn test_agent_control_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("control").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Control agent status"))
        .stdout(predicate::str::contains("active"))
        .stdout(predicate::str::contains("paused"));
}

/// Test agent invoke help.
#[test]
fn test_agent_invoke_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("invoke").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Invoke agent with input"));
}

/// Test agent memory help.
#[test]
fn test_agent_memory_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("memory").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get agent memory"));
}

/// Test agent executions help.
#[test]
fn test_agent_executions_help() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("agent").arg("executions").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Get agent execution history"));
}
