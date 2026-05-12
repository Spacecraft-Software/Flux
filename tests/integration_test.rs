// SPDX-License-Identifier: GPL-3.0-or-later

use std::process::Command;

#[test]
fn test_dns_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir("/steelbore/flux")
        .output()
        .expect("cargo run should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dns"), "Help should mention dns");
    assert!(stdout.contains("--json"), "Help should mention --json");
    assert!(
        stdout.contains("Mohamed Hammad"),
        "Help should contain maintainer"
    );
}

#[test]
fn test_dns_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .current_dir("/steelbore/flux")
        .output()
        .expect("cargo run should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"), "Version should contain 0.1.0");
}

#[test]
fn test_ai_agent_json_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "list", "--providers", "--json"])
        .current_dir("/steelbore/flux")
        .env("AI_AGENT", "1")
        .output()
        .expect("cargo run should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON envelope
    assert!(
        stdout.contains("metadata"),
        "JSON output should have metadata"
    );
    assert!(stdout.contains("data"), "JSON output should have data");
    assert!(
        stdout.contains("google"),
        "Provider list should contain google"
    );
}

#[test]
fn test_schema_emits_json() {
    let output = Command::new("cargo")
        .args(["run", "--", "schema"])
        .current_dir("/steelbore/flux")
        .env("AI_AGENT", "1")
        .output()
        .expect("cargo run should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("$schema"), "Schema should contain $schema");
    assert!(
        stdout.contains("exit_codes"),
        "Schema should contain exit_codes"
    );
}

#[test]
fn test_describe_manifest() {
    let output = Command::new("cargo")
        .args(["run", "--", "describe"])
        .current_dir("/steelbore/flux")
        .env("AI_AGENT", "1")
        .output()
        .expect("cargo run should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("flux"), "Describe should mention flux");
    assert!(
        stdout.contains("subcommands"),
        "Describe should list subcommands"
    );
}

#[test]
fn test_detect_no_panic() {
    let output = Command::new("cargo")
        .args(["run", "--", "detect", "--json"])
        .current_dir("/steelbore/flux")
        .env("AI_AGENT", "1")
        .output()
        .expect("cargo run should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should return JSON envelope even if detection fails
    assert!(
        stdout.contains("metadata"),
        "Output should have metadata envelope"
    );
}
