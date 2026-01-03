//! Build script for tss-gui.
//!
//! Captures build-time information for the About dialog:
//! - Rust version
//! - Target triple
//! - Build date

use std::process::Command;

fn main() {
    // Capture Rust version
    let rust_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=RUST_VERSION={rust_version}");

    // Capture target triple from environment
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={target}");

    // Capture build date using the `date` command (works on Unix and macOS)
    // Falls back to "unknown" on Windows or if command fails
    let build_date = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            // Try Windows date format
            Command::new("cmd")
                .args(["/C", "echo %DATE:~6,4%-%DATE:~3,2%-%DATE:~0,2%"])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

    println!("cargo:rustc-env=BUILD_DATE={build_date}");

    // Rerun if Cargo.toml changes (version might change)
    println!("cargo:rerun-if-changed=Cargo.toml");
}
