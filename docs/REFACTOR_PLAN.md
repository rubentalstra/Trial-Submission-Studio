# Refactor Plan (Clean Core, SDTM-Aligned)

This refactor is a deliberate, breaking-change cleanup. The goals are to remove
legacy surfaces, enforce strict Ports & Adapters boundaries, and align naming
with SDTM terminology where it is unambiguous.

## Goals

- Remove legacy/compatibility modules and unused pipelines.
- Eliminate barrel re-exports; always import from defining modules.
- Keep domain logic pure and I/O confined to infrastructure adapters.
- Reduce overhead by collapsing redundant adapters/wrappers.
- Use modern Python 3.14 style (slots where safe, clean typing).
- Align names with SDTM terms (domain/dataset, SUPPQUAL/RELREC, etc.).

## Non-Goals

- Preserve backwards compatibility or transitional shims.
- Broad algorithm changes or behavior changes without clear SDTM rationale.

## Step-by-Step Plan

1. Remove legacy/unused packages
   - Delete `cdisc_transpiler/services/` (progress reporting, file organization).
   - Delete `cdisc_transpiler/transformations/` (legacy pipeline).
   - Inline/move any remaining logic into a proper adapter.

2. Remove barrel re-exports
   - Strip `__init__.py` re-exports in `application/`, `domain/`,
     `infrastructure/`, and `cli/` packages.
   - Update all imports to reference defining modules directly.

3. Normalize adapter boundaries
   - Keep ports in `application/ports/*` and implementations in
     `infrastructure/*`.
   - Rename ambiguous classes to `*Adapter`/`*Repository`/`*Writer` where
     appropriate.

4. Modernize data models
   - Add `slots=True` to dataclasses where safe.
   - Tighten typing in core DTOs/entities.

5. SDTM naming alignment
   - Prefer SDTM terms (Domain/Dataset, SUPPQUAL, RELREC, Trial Design, etc.).
   - Rename internal components where names conflict with SDTM usage.

6. Docs update
   - Update `README.md` and `docs/ARCHITECTURE.md` to reflect the new
     clean-core layout and the removal of legacy packages.

7. Health checks
   - Run `pyright` (and formatter) after refactor stages.

## Status

- Completed: Steps 1-4
- In progress: Step 5 (SDTM naming alignment)
- Pending: Steps 6-7
