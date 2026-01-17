//! GitHub API client and types.
//!
//! This module provides a client for interacting with the GitHub Releases API.

pub mod client;
pub mod types;

pub use client::GitHubClient;
pub use types::{GitHubAsset, GitHubRelease};
