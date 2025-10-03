//! Integration test wrapper that invokes the example regression harness.
//! This allows `cargo test` to execute the same golden comparison used in CI.
use std::process::Command;

#[test]
fn examples_harness() {
    // Run the harness from the repository root (two levels up from this file).
    // We rely on relative path: crate root is rust/tong, go up two to repo root.
    let status = Command::new("bash")
        .arg("../../scripts/check_examples.sh")
        .status()
        .expect("failed to run example harness script");
    assert!(status.success(), "example harness failed");
}