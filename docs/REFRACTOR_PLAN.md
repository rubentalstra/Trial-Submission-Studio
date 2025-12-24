# CDISC Transpiler (Rust) — End-to-End Refactor/Rebuild Plan

## Executive Summary

### Goals

- Deliver a **100% Rust** CLI tool that runs **fully offline** and transpiles
  clinical trial source data into **CDISC SDTM** outputs.
- Produce correct, deterministic, auditable outputs:
  - **SAS V5 Transport (XPT)**
  - **Dataset-XML v1.0**
  - **Define-XML v2.1**
  - Optional **SAS exports/scripts** (as outputs only, not required for
    correctness).
- Make **validation first-class**: conformance, CT, and XML schema checks run as
  explicit pipeline stages and **fail CI** on violations.
- Keep architecture **lean**: clear boundaries, dependency direction, typed
  components, and minimal dependencies.

### Constraints (non-negotiable)

- **Language**: end-state is **Rust-only**. No Python runtime, no Python
  bridges, no “call Python from Rust”.
- **Offline runtime**: no network access, no runtime fetching of CT/standards.
  All standards + CT + assets are committed.
- **Standards-driven**: SDTM v2.0 + SDTMIG v3.4 + Pinnacle 21 `Rules.csv` +
  Define-XML 2.1 + Dataset-XML 1.0.
- **Output correctness**:
  - deterministic mapping
  - full provenance (source → derived SDTM)
  - validation gates that fail the build when violated
- **Lean architecture**:
  - mapping and validation must be unit-testable
  - writers contain **no mapping logic**
  - dependencies must be justified

### Non-goals (explicit)

- Online CT or standards updates.
- “Auto-magic” mapping driven by LLMs or network services.
- Perfect support for every sponsor’s custom nuances on day one.
- Building an interactive UI; the primary product is a CLI.

---

## Current State Assessment (Repo Deep Dive)

This repository is currently a **Python** implementation with a ports/adapters
flavor.
