//! Build script for tss-gui.
//!
//! Captures build-time information for the About dialog:
//! - Rust version
//! - Target triple
//! - Build date
//! - Build number (TSS-X.Y for CI, LOCAL.Y for local builds)
//!
//! On Windows, also embeds:
//! - Application icon
//! - Version info (shown in File Properties)

use std::process::Command;

fn main() {
    // Capture Rust version
    let rust_version = get_rust_version();
    println!("cargo:rustc-env=RUST_VERSION={rust_version}");

    // Capture target triple from environment
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={target}");

    // Capture build date
    let build_date = get_build_date();
    println!("cargo:rustc-env=BUILD_DATE={build_date}");

    // Capture git commit count for build number
    let commit_count = get_git_commit_count();

    // Generate build number: TSS-{run}.{commits} for CI, LOCAL.{commits} for local
    let build_number = get_build_number(&commit_count);
    println!("cargo:rustc-env=BUILD_NUMBER={build_number}");

    // Rerun triggers
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=TSS_BUILD_NUMBER");
    println!("cargo:rerun-if-env-changed=GITHUB_ACTIONS");
    println!("cargo:rerun-if-env-changed=GITHUB_RUN_NUMBER");

    // Windows: embed icon and version info
    #[cfg(target_os = "windows")]
    embed_windows_resources(&build_number, &build_date, &commit_count);
}

/// Get the Rust compiler version.
fn get_rust_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get the build date in YYYY-MM-DD format.
fn get_build_date() -> String {
    // Try Unix date command first (macOS, Linux)
    Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .or_else(|| {
            // Windows fallback using PowerShell
            Command::new("powershell")
                .args(["-Command", "Get-Date -Format 'yyyy-MM-dd'"])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get the total git commit count.
fn get_git_commit_count() -> String {
    Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "0".to_string())
}

/// Generate the build number based on environment.
///
/// Priority:
/// 1. `TSS_BUILD_NUMBER` env var (explicit override from CI)
/// 2. GitHub Actions: `TSS-{GITHUB_RUN_NUMBER}.{commit_count}`
/// 3. Local development: `LOCAL.{commit_count}`
fn get_build_number(commit_count: &str) -> String {
    // Priority 1: Explicit build number from CI workflow
    if let Ok(bn) = std::env::var("TSS_BUILD_NUMBER") {
        return bn;
    }

    // Priority 2: GitHub Actions environment - auto-generate
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        let run_number = std::env::var("GITHUB_RUN_NUMBER").unwrap_or_else(|_| "0".to_string());
        return format!("TSS-{run_number}.{commit_count}");
    }

    // Priority 3: Local development build
    format!("LOCAL.{commit_count}")
}

/// Embed Windows resources (icon, version info) into the executable.
#[cfg(target_os = "windows")]
fn embed_windows_resources(build_number: &str, build_date: &str, commit_count: &str) {
    let mut res = winresource::WindowsResource::new();

    // Set application icon
    res.set_icon("../../packaging/windows/icon.ico");

    // Parse version from CARGO_PKG_VERSION (e.g., "0.0.1-alpha.4")
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let (major, minor, patch) = parse_semver(&version);
    let build: u64 = commit_count.parse().unwrap_or(0);

    // Windows VERSIONINFO requires numeric version: MAJOR.MINOR.PATCH.BUILD
    // Each component is 16-bit, packed into 64-bit value
    let file_version = ((major as u64) << 48)
        | ((minor as u64) << 32)
        | ((patch as u64) << 16)
        | (build & 0xFFFF);

    res.set_version_info(winresource::VersionInfo::FILEVERSION, file_version);
    res.set_version_info(winresource::VersionInfo::PRODUCTVERSION, file_version);

    // String version info (shown in Windows Explorer Properties > Details)
    res.set("FileDescription", "Trial Submission Studio - Clinical Trial Data Management");
    res.set("ProductName", "Trial Submission Studio");
    res.set("ProductVersion", &format!("{version} ({build_number})"));
    res.set(
        "FileVersion",
        &format!("{major}.{minor}.{patch}.{build}"),
    );
    res.set("OriginalFilename", "trial-submission-studio.exe");
    res.set("LegalCopyright", "Copyright (c) 2024-2026 Ruben Talstra");
    res.set("CompanyName", "Ruben Talstra");
    res.set("Comments", &format!("Build: {build_number} ({build_date})"));

    if let Err(e) = res.compile() {
        println!("cargo:warning=Failed to compile Windows resources: {e}");
    }
}

/// Parse semantic version string, extracting major.minor.patch.
/// Handles pre-release suffixes like "0.0.1-alpha.4".
#[cfg(target_os = "windows")]
fn parse_semver(version: &str) -> (u16, u16, u16) {
    // Strip pre-release suffix (e.g., "-alpha.4")
    let version_core = version.split('-').next().unwrap_or("0.0.0");
    let parts: Vec<u16> = version_core
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}
