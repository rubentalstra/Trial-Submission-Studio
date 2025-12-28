# CDISC Transpiler (Rust)

A Rust-first CLI tool for transpiling clinical trial source data into CDISC SDTM
outputs (XPT, Dataset-XML, Define-XML; SAS scripts deferred in v1) with strict,
offline validation.

## Status

- Rust rebuild is in progress; see `docs/REFRACTOR_PLAN.md`.
- The existing Python CLI remains the functional reference until parity is
  achieved.
- Task tracker: `docs/RUST_CLI_TASKS.md`.

## Target Features

- Fully offline operation with committed standards and CT
- Deterministic, auditable output generation
- Validation-first pipeline with conformance gating
- Outputs: XPT (SAS V5), Dataset-XML 1.0, Define-XML 2.1, SAS deferred in v1

## Rust CLI (Planned Interface)

```bash
cdisc-transpiler study <study_folder> [options]
cdisc-transpiler domains
```

Minimal options (v1):

- `--output-dir`
- `--format [xpt|xml|both]`
- `-v` / `-vv` verbosity, `-q` / `-qq` quieter
- `--log-level [error|warn|info|debug|trace]`
- `--log-format [pretty|compact|json]`
- `--color [auto|always|never]`
- `--log-file <path>`

No config file in v1; defaults are compiled.

## Logging and PHI

- Logs avoid row-level values; only counts and metadata are logged.
- Prefer `--log-level` (or `-v`/`-vv`) for verbosity control; `--quiet` limits
  logs to errors.
- Use `--log-format` for JSON/compact output, `--color` to control ANSI output,
  and `--log-file` to persist logs.

## Legacy Python CLI (Current Implementation)

If you need a working CLI right now, the Python tool is available:

```bash
# Create and activate virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install with dev dependencies
pip install -e .[dev]

# Run
cdisc-transpiler study mockdata/DEMO_GDISC_20240903_072908/
cdisc-transpiler domains
```

## Project Docs

- Strategy and architecture: `docs/REFRACTOR_PLAN.md`
- Task tracker: `docs/RUST_CLI_TASKS.md`
- Standards assets: `standards/` (offline, committed source of truth)

## References

Record Layout of a SAS® Version 5 or 6 Data Set in SAS® Transport (Xport) Format

- https://support.sas.com/content/dam/SAS/support/en/technical-papers/record-layout-of-a-sas-version-5-or-6-data-set-in-sas-transport-xport-format.pdf
