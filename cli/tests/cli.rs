use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_cli_sample_command() {
    let mut cmd = Command::cargo_bin("cli").unwrap();
    
    // Test the `sample` command
    let assert = cmd.arg("sample").assert();
    
    // It should succeed and output valid JSON
    assert.success()
        .stdout(predicate::str::contains("ComputeBudget"))
        .stdout(predicate::str::contains("priority_fee_microlamports"));
}

#[test]
fn test_cli_predict_with_missing_file() {
    let mut cmd = Command::cargo_bin("cli").unwrap();
    
    // Test the `predict` command with a non-existent file
    let assert = cmd.arg("predict").arg("--tx-file").arg("does_not_exist.json").assert();
    
    // It should fail and complain about the file (either rust IO error or clap error)
    assert.failure();
}

// We don't want to run a full predict against mainnet in a standard unit test because it's flaky and network-dependent.
// However, we can create a temporary sample file and run it against a dummy local RPC URL (which will fail gracefully at the RPC fetch step).
#[test]
fn test_cli_predict_graceful_rpc_failure() {
    // 1. Generate sample output to a temp file
    let temp_file = "temp_test_tx.json";
    
    let mut sample_cmd = Command::cargo_bin("cli").unwrap();
    let output = sample_cmd.arg("sample").output().unwrap();
    fs::write(temp_file, output.stdout).unwrap();

    // 2. Run predict against a bad RPC URL
    let mut predict_cmd = Command::cargo_bin("cli").unwrap();
    let assert = predict_cmd
        .arg("predict")
        .arg("--tx-file").arg(temp_file)
        .arg("--rpc-url").arg("http://127.0.0.1:9999") // Should be dead
        .assert();

    // It should succeed in execution (exit code 0), but print a failure message about fetching network state.
    assert.success()
        .stdout(predicate::str::contains("Failed to fetch network state"));

    // Cleanup
    let _ = fs::remove_file(temp_file);
}
