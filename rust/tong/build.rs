use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Skip build-time commands for WASM target
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("wasm") {
        println!("cargo:rustc-env=GIT_HASH=wasm-build");
        println!("cargo:rustc-env=GIT_DIRTY=unknown");
        println!("cargo:rustc-env=BUILD_UNIX=0");
        println!("cargo:rustc-env=BUILD_TIME=wasm");
        return;
    }

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
    // Build time in YYYYmmdd_HHMM format - use chrono or fallback for cross-platform
    #[cfg(unix)]
    let datetime = Command::new("date")
        .arg("+%Y%m%d_%H%M")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    #[cfg(not(unix))]
    let datetime = format!("{}", ts); // Fallback to timestamp on Windows
    
    println!("cargo:rustc-env=BUILD_TIME={}", datetime);
}
