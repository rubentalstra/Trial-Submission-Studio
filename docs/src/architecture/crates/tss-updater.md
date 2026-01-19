# tss-updater

Application update mechanism crate.

## Overview

`tss-updater` checks for and applies application updates from GitHub releases.

## Responsibilities

- Check for new versions
- Download updates
- Verify checksums
- Apply updates (platform-specific)

## Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
semver = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
```

## Architecture

### Module Structure

```
tss-updater/
├── src/
│   ├── lib.rs
│   ├── checker.rs       # Version checking
│   ├── downloader.rs    # Download handling
│   ├── verifier.rs      # Checksum verification
│   └── installer.rs     # Update installation
```

## Update Flow

```
┌─────────────────┐
│ Check Version   │
│ (GitHub API)    │
└────────┬────────┘
         │ New version?
         ▼
┌─────────────────┐
│ Download Asset  │
│ (Release file)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Verify Checksum │
│ (SHA256)        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Install Update  │
│ (Platform)      │
└─────────────────┘
```

## API

### Checking for Updates

```rust
use tss_updater::{UpdateChecker, UpdateInfo};

let checker = UpdateChecker::new("rubentalstra", "Trial-Submission-Studio");

match checker.check_for_updates(current_version)? {
Some(update) => {
println ! ("New version available: {}", update.version);
println ! ("Release notes: {}", update.notes);
}
None => {
println ! ("You're up to date!");
}
}
```

### Update Info

```rust
pub struct UpdateInfo {
    pub version: Version,
    pub notes: String,
    pub download_url: String,
    pub checksum_url: String,
    pub published_at: DateTime<Utc>,
}
```

### Downloading

```rust
use tss_updater::Downloader;

let downloader = Downloader::new();
let progress_callback = | percent| {
println ! ("Download: {}%", percent);
};

downloader.download( & update.download_url, & temp_path, progress_callback) ?;
```

### Verification

```rust
use tss_updater::Verifier;

let verifier = Verifier::new();
let expected_hash = verifier.fetch_checksum( & update.checksum_url) ?;

if verifier.verify_file( & temp_path, & expected_hash)? {
println ! ("Checksum verified!");
} else {
return Err(UpdateError::ChecksumMismatch);
}
```

## Platform-Specific Installation

### macOS

Uses `tss-updater-helper` for atomic bundle swap:

1. Download new app bundle to temp directory
2. Spawn `tss-updater-helper` with config
3. Main app exits
4. Helper performs atomic swap and relaunches

See [tss-updater-helper](tss-updater-helper.md) for details.

### Windows

1. Extract to temp location
2. Schedule replacement on restart
3. Restart application

### Linux

1. Extract new binary
2. Replace existing binary
3. Restart application

## Security

### HTTPS Only

All connections use HTTPS:

- GitHub API
- Release downloads
- Checksum files

### Checksum Verification

SHA256 checksums verified before installation.

### Signed Releases

(Future) Code signing verification for releases.

## Configuration

### Update Settings

```rust
pub struct UpdateConfig {
    pub check_on_startup: bool,
    pub auto_download: bool,
    pub prerelease: bool,  // Include prereleases
}
```

### Default Behavior

- Check on startup (with delay)
- Notify user, don't auto-install
- Stable releases only

## Error Handling

```rust
#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("Installation failed: {0}")]
    InstallFailed(String),
}
```

## Testing

```bash
cargo test --package tss-updater
```

### Test Strategy

- Mock HTTP responses
- Checksum calculation tests
- Version comparison tests

## See Also

- [Architecture Overview](../overview.md) - System design
- [tss-gui](tss-gui.md) - UI integration
- [tss-updater-helper](tss-updater-helper.md) - macOS update helper
