# Phase 0 Bootstrap (Rust + Offline Standards)

This repo is bootstrapping a **100% Rust**, **fully offline** SDTM transpiler.

## Rust policy

- **Edition**: Rust 2024
- **Toolchain pin**: see [../rust-toolchain.toml](../rust-toolchain.toml)
- **MSRV**: `1.92.0` (matches the pinned toolchain)

Policy: bump MSRV intentionally (e.g., quarterly). CI should remain green on the pinned toolchain.

## Standards registry

All standards/CT assets live under [../standards](../standards) and are pinned by checksums in:

- [../standards/manifest.toml](../standards/manifest.toml)

The manifest is the single source of truth for:

- Version pins (SDTM/SDTMIG/CT/Conformance Rules)
- File inventory (paths relative to `standards/`)
- SHA-256 checksums over raw bytes

## Commands

Run from repo root:

- Verify manifest integrity and CSV parseability:
  - `cargo run -p sdtm-cli -- standards verify`
- Print quick counts and pins:
  - `cargo run -p sdtm-cli -- standards summary`
- Emit machine-readable JSON report:
  - `cargo run -p sdtm-cli -- standards doctor --json -`
  - `cargo run -p sdtm-cli -- standards doctor --json out.json`

## Updating standards files and manifest

When a standards asset changes (or you add a new one):

1. Put the file under the correct `standards/...` path.
2. Compute SHA-256 over raw bytes:
   - `shasum -a 256 standards/<path>`
3. Update the corresponding `[[files]]` entry in [../standards/manifest.toml](../standards/manifest.toml).
4. Run:
   - `cargo run -p sdtm-cli -- standards verify`

If you add a *new* file, add a new `[[files]]` entry with:

- `path` (relative to `standards/`, use forward slashes)
- `sha256` (64 lowercase hex)
- `kind` (csv/toml/xsl/pdf/other)
- `role` (choose a stable role name; required roles are enforced)
