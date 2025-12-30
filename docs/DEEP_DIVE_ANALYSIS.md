# Deep Dive Analysis & Simplification Plan

## Executive Summary

This document analyzes the current state of `sdtm-core` and `sdtm-normalization`
to identify opportunities for aggressive code removal and simplification. The
goal is to eliminate "double" logic (legacy CLI infrastructure) and streamline
the transformation pipeline for the GUI.

## Crate Analysis

### 1. `sdtm-core`

This crate currently acts as the orchestration engine. It contains a heavy
"Processor Registry" pattern that may be over-engineered for the current needs.

#### File Descriptions

- **`src/lib.rs`**: Entry point. Defines the module structure.
- **`src/pipeline_context.rs`**: Holds global state (`StudyMetadata`,
  `ProcessingOptions`, `CodeList`). **Keep**, but simplify if possible.
- **`src/processor.rs`**: Contains `ProcessorRegistry` and `process_domain`.
  This is the primary source of "double" logic. It dynamically dispatches to
  domain processors. **Candidate for Removal/Simplification**.
- **`src/transforms.rs`**: Contains generic transformation logic
  (`apply_transform`). **Keep**, but ensure it's not duplicating
  `sdtm-normalization`.
- **`src/domain_processors/mod.rs`**: Module definition for all domain
  processors.
- **`src/domain_processors/processor_trait.rs`**: Defines the `DomainProcessor`
  trait. **Candidate for Removal** if we move to a functional approach.
- **`src/domain_processors/operations.rs`**: Shared operations (`backward_fill`,
  `clear_unit_when_empty`). **Keep**, these are useful helpers.
- **`src/domain_processors/common.rs`**: Common utilities (`has_column`,
  `string_column`). **Keep**.
- **`src/domain_processors/[domain].rs`** (e.g., `ae.rs`, `dm.rs`): Individual
  domain logic. These contain specific business rules. **Keep**, but refactor to
  be simple functions rather than struct implementations if possible.

### 2. `sdtm-normalization`

This crate provides reusable transformation utilities. It seems to overlap
slightly with `sdtm-core`'s `transforms.rs`.

#### File Descriptions

- **`src/lib.rs`**: Entry point.
- **`src/normalization/ct.rs`**: Controlled Terminology normalization logic.
  **Keep**.
- **`src/normalization/datetime.rs`**: ISO 8601 parsing/formatting. **Keep**.
- **`src/normalization/numeric.rs`**: Numeric conversions. **Keep**.
- **`src/frame_builder.rs`**: CSV to DataFrame conversion. **Keep**.
- **`src/suppqual.rs`**: Logic for generating SUPP-- datasets. **Keep**.
- **`src/relationships.rs`**: Logic for generating RELREC, etc. **Keep**.

## Redundancy Analysis ("Double Logic")

The primary redundancy is the **Processor Registry** pattern in `sdtm-core`.

- **Current State**: `sdtm-core` registers a processor for every domain string
  ("DM" -> `DmProcessor`). This requires a lot of boilerplate
  (`default_registry`, `lazy_static` or similar initialization).
- **Desired State**: A simple function call or a match statement. If the GUI
  knows it's processing "DM", it should just call `process_dm()`. We don't need
  a dynamic registry if we are just running a pipeline.

## Simplification Plan

1. **Remove `ProcessorRegistry`**: Delete the dynamic dispatch mechanism in
   `sdtm-core/src/processor.rs`.
2. **Flatten Domain Processors**: Convert `struct DmProcessor` into a simple
   function `pub fn process_dm(...)`.
3. **Consolidate Transformations**: Move any generic transformations from
   `sdtm-core/src/transforms.rs` to `sdtm-normalization` or a new
   `sdtm-transform` crate if it exists, or just keep them in `sdtm-core` but
   make them pure functions.
4. **Direct Invocation**: The pipeline should simply iterate over the requested
   domains and call the corresponding function directly.

## Action Items

1. [ ] Refactor `sdtm-core/src/domain_processors/*.rs` to export public
       functions instead of implementing a trait.
2. [ ] Delete `sdtm-core/src/processor.rs` (the registry).
3. [ ] Delete `sdtm-core/src/domain_processors/processor_trait.rs`.
4. [ ] Update `sdtm-core/src/lib.rs` to expose domain functions directly.
