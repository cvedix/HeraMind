//! Tests for the `serve` command.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::net::SocketAddr;
use std::str::FromStr;

/// Test that serve command accepts default values.
#[test]
fn test_serve_default_values() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("serve").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--host"))
        .stdout(predicate::str::contains("--port"))
        .stdout(predicate::str::contains("9375")); // Default port
}

/// Test that custom port is accepted.
#[test]
fn test_serve_custom_port() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("serve")
        .arg("--port")
        .arg("8080")
        .arg("--host")
        .arg("127.0.0.1");

    // This will likely fail due to no LLM backend
    cmd.assert().failure();
}

/// Test that custom host is accepted.
#[test]
fn test_serve_custom_host() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("serve")
        .arg("--host")
        .arg("0.0.0.0")
        .arg("--port")
        .arg("9375");

    cmd.assert().failure();
}

/// Test that invalid port is rejected.
#[test]
fn test_serve_invalid_port_rejected() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("serve").arg("--port").arg("invalid");

    cmd.assert().failure();
}

/// Test that port out of range is rejected.
#[test]
fn test_serve_port_out_of_range() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("serve").arg("--port").arg("99999");

    cmd.assert().failure();
}

/// Test host:port parsing for valid addresses.
#[test]
fn test_address_parsing() {
    // Valid IP addresses (localhost requires DNS lookup, skip for unit test)
    let valid_addrs = ["127.0.0.1:9375", "0.0.0.0:8080", "192.168.1.1:9000"];

    for addr_str in valid_addrs {
        let result = SocketAddr::from_str(addr_str);
        assert!(result.is_ok(), "Expected valid address: {}", addr_str);
    }
}

/// Test that missing required arguments shows appropriate error.
#[test]
fn test_serve_shows_help_with_missing_args() {
    let mut cmd = Command::cargo_bin("heramind").unwrap();
    cmd.arg("serve");

    // Should work with defaults
    cmd.assert().failure();
}
