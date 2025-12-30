# Deep Dive Analysis & Simplification Plan

## Executive Summary

This document analyzes the current state of the entire workspace to identify
opportunities for aggressive code removal and simplification. The goal is to
eliminate "double" logic (legacy CLI infrastructure), streamline the
transformation pipeline for the GUI, and ensure a clear separation of concerns.

## Crate Analysis

### 1. `sdtm-core`

**Role**: Orchestration Engine & Domain Logic **Status**: **Major Refactor
Target**

This crate currently acts as the orchestration engine. It contains a heavy
"Processor Registry" pattern that is over-engineered for the current needs.

#### File Descriptions

- **`src/lib.rs`**: Entry point. Defines the module structure.
- **`src/pipeline_context.rs`**: Holds global state (`StudyMetadata`,
  `ProcessingOptions`, `CodeList`). **Keep**, but simplify.
- **`src/processor.rs`**: Contains `ProcessorRegistry` and `process_domain`.
  This is the primary source of "double" logic. It dynamically dispatches to
  domain processors. **Candidate for Removal/Simplification**.
- **`src/transforms.rs`**: Contains generic transformation logic
  (`apply_transform`). **Keep**, but ensure it's not duplicating
  `sdtm-transform`.
- **`src/domain_processors/mod.rs`**: Module definition for all domain
  processors.
- **`src/domain_processors/processor_trait.rs`**: Defines the `DomainProcessor`
  trait. **Candidate for Removal** in favor of functional approach.
- **`src/domain_processors/operations.rs`**: Shared operations (`backward_fill`,
  `clear_unit_when_empty`). **Keep**, these are useful helpers.
- **`src/domain_processors/common.rs`**: Common utilities (`has_column`,
  `string_column`). **Keep**.
- **`src/domain_processors/[domain].rs`** (e.g., `ae.rs`, `dm.rs`): Individual
  domain logic. These contain specific business rules. **Keep**, but refactor to
  be simple functions.

### 2. `sdtm-transform` (formerly `sdtm-normalization`)

**Role**: Reusable Transformation Utilities **Status**: **Stable / Keep**

**Naming Decision**: Renamed from `sdtm-normalization` to `sdtm-transform`.
_Reasoning_: This crate does more than just "normalize" values (clean-up). It
performs structural transformations like generating Supplemental Qualifiers
(`suppqual.rs`) and Relationship datasets (`relationships.rs`). "Transformation"
is a broader term that correctly encompasses both value normalization and
structural reshaping.

#### File Descriptions

- **`src/lib.rs`**: Entry point.
- **`src/normalization/ct.rs`**: Controlled Terminology normalization logic.
  **Keep**.
- **`src/normalization/datetime.rs`**: ISO 8601 parsing/formatting. **Keep**.
- **`src/normalization/numeric.rs`**: Numeric conversions. **Keep**.
- **`src/frame_builder.rs`**: CSV to DataFrame conversion. **Keep**.
- **`src/suppqual.rs`**: Logic for generating SUPP-- datasets. **Keep**.
- **`src/relationships.rs`**: Logic for generating RELREC, etc. **Keep**.

### 3. `sdtm-gui`

**Role**: Desktop Application (Frontend) **Status**: **Active Development**

The user interface built with `eframe`/`egui`.

#### File Descriptions

- **`src/main.rs`**: Application entry point. Sets up the window and logging.
- **`src/app.rs`**: Main application state and update loop.
- **`src/services/`**: Business logic bridges (e.g., `study_loader.rs`).
- **`src/state/`**: Application state management.
- **`src/views/`**: UI components and screens.

### 4. `sdtm-ingest`

**Role**: Data Loading & Discovery **Status**: **Review for Redundancy**

Handles reading CSV files and discovering study structure.

#### File Descriptions

- **`src/lib.rs`**: Entry point.
- **`src/csv_table.rs`**: CSV reading logic using Polars. **Keep**.
- **`src/discovery.rs`**: File system scanning to find domains. **Keep**.
- **`src/study_metadata.rs`**: **CRITICAL CHECK**. This file likely contains
  logic that overlaps with the new `sdtm-model` centralization. It should only
  contain _loading_ logic, not type definitions.
- **`src/polars_utils.rs`**: Low-level Polars helpers. **Keep**.

### 5. `sdtm-map`

**Role**: Column Mapping Engine **Status**: **Stable**

Provides fuzzy matching to map source columns to SDTM variables.

#### File Descriptions

- **`src/engine.rs`**: The core scoring algorithm (Jaro-Winkler, etc.).
  **Keep**.
- **`src/patterns.rs`**: Regex patterns and synonym maps. **Keep**.
- **`src/repository.rs`**: Persistence for mapping configurations. **Keep**.

### 6. `sdtm-model`

**Role**: Core Types & Data Model **Status**: **Central Source of Truth**

Recently refactored to hold all shared types.

#### File Descriptions

- **`src/domain.rs`**: `Domain`, `Variable` definitions.
- **`src/metadata.rs`**: `StudyMetadata`, `SourceColumn`.
- **`src/options.rs`**: `ProcessingOptions`.
- **`src/ct.rs`**: Controlled Terminology types.
- **`src/conformance.rs`**: Validation types.

### 7. `sdtm-report`

**Role**: Output Generation **Status**: **Stable**

Generates the final artifacts.

#### File Descriptions

- **`src/xpt.rs`**: High-level XPT generation logic.
- **`src/dataset_xml.rs`**: Dataset-XML generation.
- **`src/define_xml.rs`**: Define-XML generation.
- **`src/sas.rs`**: SAS program generation.

### 8. `sdtm-standards`

**Role**: Static Data Loader **Status**: **Stable**

Loads the offline standards (SDTMIG, CT, P21 Rules).

#### File Descriptions

- **`src/loaders.rs`**: Loads SDTMIG CSVs.
- **`src/ct_loader.rs`**: Loads CT CSVs.
- **`src/p21_loader.rs`**: Loads Pinnacle 21 rules.

### 9. `sdtm-validate`

**Role**: Conformance Checking **Status**: **Stable**

Validates the final DataFrames against rules.

#### File Descriptions

- **`src/lib.rs`**: Main validation logic (CT, ISO8601, Required vars).

### 10. `sdtm-xpt`

**Role**: Low-level XPT I/O **Status**: **Stable**

A specialized crate for reading/writing the binary XPT format.

## Redundancy Analysis ("Double Logic")

1. **Processor Registry (`sdtm-core`)**: The dynamic dispatch system is
   unnecessary complexity.
   - _Action_: Replace with direct function calls.
2. **Study Metadata (`sdtm-ingest` vs `sdtm-model`)**: `sdtm-ingest` might still
   be defining types that are now in `sdtm-model`.
   - _Action_: Verify `sdtm-ingest/src/study_metadata.rs` only _uses_ types from
     `sdtm-model`.
3. **Transformation Overlap**: `sdtm-core/src/transforms.rs` vs
   `sdtm-transform`.
   - _Action_: Audit `transforms.rs` and move unique logic to `sdtm-transform`
     or `sdtm-core/src/domain_processors/common.rs`.

## Type Standardization & Naming Convention

To ensure consistency across the workspace and alignment with SDTMIG v3.4, we
will adopt the following naming conventions.

### 1. SDTM Target Types (sdtm-model)

These types represent the _target_ SDTM structure (the output). They must
strictly follow SDTMIG terminology.

- **`Domain`**: Represents a domain definition (e.g., "AE", "DM"). **Keep**.
- **`Variable`**: Represents a column definition within a domain. **Keep**.
- **`DatasetClass`**: Represents the observation class (Interventions, Events,
  Findings). **Keep**.
- **`Codelist`**: Represents a CDISC Controlled Terminology codelist. **Keep**.
- **`Term`**: Represents a single term within a codelist. **Keep**.

### 2. Source Input Types (sdtm-model / sdtm-ingest)

These types represent the _source_ data (the input CSVs or raw data). They
should be clearly distinguished from SDTM types using the `Source` prefix.

- **`StudyMetadata`** -> **`SourceMetadata`**: Represents the metadata of the
  _source_ study (e.g., raw CSV columns).
  - _Reason_: Avoid confusion with SDTM "Study Metadata" (Define-XML).
- **`SourceColumn`**: Represents a column in the source data. **Keep**.
- **`StudyCodelist`** -> **`SourceCodelist`**: Represents a format/codelist in
  the source data.
  - _Reason_: Consistency with `SourceMetadata`.

### 3. Configuration Types (sdtm-model)

- **`ProcessingOptions`**: Configuration for the transformation pipeline.
  **Keep**.
- **`NormalizationOptions`**: Configuration for value normalization. **Keep**.

## Action Items

1. [ ] **sdtm-core**: Refactor `domain_processors/*.rs` to export public
       functions.
2. [ ] **sdtm-core**: Delete `processor.rs` and `processor_trait.rs`.
3. [ ] **sdtm-core**: Update `lib.rs` to expose domain functions.
4. [ ] **sdtm-ingest**: Refactor `study_metadata.rs` to remove duplicate type
       definitions if any.
5. [ ] **sdtm-model**: Rename `StudyMetadata` to `SourceMetadata`.
6. [ ] **sdtm-model**: Rename `StudyCodelist` to `SourceCodelist`.
