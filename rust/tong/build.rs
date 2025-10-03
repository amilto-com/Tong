use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Git hash (short)
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });
    if let Some(h) = git_hash {
        println!("cargo:rustc-env=GIT_HASH={}", h);
    }
    // Dirty flag
    let dirty = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map(|s| if s.success() { "clean" } else { "dirty" })
        .unwrap_or("unknown");
    println!("cargo:rustc-env=GIT_DIRTY={}", dirty);
    // Build timestamp (unix seconds)
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("cargo:rustc-env=BUILD_UNIX={}", ts);
}
