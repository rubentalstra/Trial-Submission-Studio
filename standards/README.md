# standards/

This folder contains **offline, version-pinned** standards assets used by the
Rust SDTM transpiler.

## Key properties

- No runtime downloads or web fetching.
- All files are addressed through [manifest.toml](manifest.toml) and verified
  via `sdtm standards verify`.
- Checksums are **SHA-256 over raw bytes** as stored in git.

## Layout

- `sdtm/` — SDTM model metadata (CSV)
- `sdtmig/` — SDTMIG implementation guidance metadata (CSV + optional PDF)
- `ct/` — Controlled Terminology (CSV), by date
- `conformance_rules/` — local conformance rule corpus (Phase 0 scaffolding)
- `xsl/` — XSL stylesheets used for Define-XML rendering (later)
