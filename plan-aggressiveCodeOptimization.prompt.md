# Detailed Aggressive Code Optimization & Reduction Plan

This plan breaks down the optimization strategy into actionable, step-by-step tasks. It prioritizes memory reduction (Phase 1) and execution speed (Phase 2), followed by architectural cleanup.

## Phase 1: `sdtm-ingest` Overhaul (Critical Memory Fix)
**Goal:** Eliminate `CsvTable` (`Vec<Vec<String>>`) and use `polars::DataFrame` as the primary data carrier from the moment files are read.

### 1.1. Refactor `sdtm-ingest/src/csv_table.rs`
-   **Task:** Replace manual CSV parsing with Polars native CSV reader.
-   **Changes:**
    -   Remove `struct CsvTable`.
    -   Remove `read_csv_rows_internal`, `normalize_header`, `normalize_cell`.
    -   Rewrite `read_csv_table` signature:
        ```rust
        // Old
        pub fn read_csv_table(path: &Path) -> Result<CsvTable>
        // New
        pub fn read_csv_table(path: &Path) -> Result<DataFrame>
        ```
    -   **Implementation Detail:**
        ```rust
        CsvReadOptions::default()
            .with_has_header(true)
            .with_infer_schema_length(Some(100)) // Only scan 100 rows for types
            .with_ignore_errors(true)            // Robustness
            .try_into_reader_with_file_path(Some(path.into()))?
            .finish()
        ```
    -   **Handling Headers:** Polars handles headers automatically. If we need normalization (trimming spaces), do it via `df.set_column_names(...)` after load.

### 1.2. Update `sdtm-core/src/frame_builder.rs`
-   **Task:** Adapt the frame builder to accept `DataFrame` instead of `CsvTable`.
-   **Changes:**
    -   The `build_frame` function currently iterates over `CsvTable.rows`.
    -   Change it to take `DataFrame` as input.
    -   If `build_frame` was doing type conversion, let Polars handle it or use `df.cast()`.

### 1.3. Cleanup `sdtm-ingest/src/lib.rs`
-   **Task:** Remove exported legacy types.
-   **Changes:**
    -   Remove `pub use csv_table::{CsvTable, ...}`.
    -   Export the new `read_csv_table`.

## Phase 2: `sdtm-core` Vectorization (Speedup)
**Goal:** Replace row-by-row iteration with Polars Expressions (`Expr`). This moves loops from "slow" Rust iteration over `AnyValue` to optimized internal loops.

### 2.1. Vectorize CT Normalization (`sdtm-core/src/processor.rs`)
-   **Task:** Rewrite `normalize_ct_columns`.
-   **Current Bottleneck:** `for idx in 0..row_count` loop with `any_to_string`.
-   **New Approach:**
    -   Create a closure or function that takes a `&Series` and returns a `Series`.
    -   Use `series.str().apply_custom` (or `map_elements`) to apply the CT lookup.
    -   **Optimization:** Pre-compile the `CaseInsensitiveSet` or lookup map *once* before the apply, not inside the loop.
    -   **Code Concept:**
        ```rust
        df.with_column(
            col(col_name).map_elements(move |s: &str| {
                ct.lookup(s).unwrap_or(s)
            }, GetOutput::from_type(DataType::String))
        )?;
        ```

### 2.2. Vectorize Base Rules
-   **Task:** Rewrite `apply_base_rules`.
-   **Changes:**
    -   **USUBJID Construction:**
        ```rust
        // Replace manual concatenation loop with:
        let expr = col("STUDYID") + lit("-") + col("SUBJID");
        df.with_column(expr.alias("USUBJID"))?;
        ```
    -   **Sequence Numbers:** Use `col("USUBJID").cumcount() + 1` (grouped by USUBJID) to generate sequences if applicable, instead of manual counters.

### 2.3. Remove `AnyValue` Conversions
-   **Task:** Audit `sdtm-core` for `any_to_string`.
-   **Action:** Replace with `series.str()` accessors which give `ChunkedArray<Utf8Type>`. This avoids allocating a `String` for every cell just to read it.

## Phase 3: Shared State & Memory (Architecture)
**Goal:** Reduce cloning of large static datasets (Standards & CT).

### 3.1. Shared Registry (`sdtm-standards`)
-   **Task:** Wrap `TerminologyRegistry` in `Arc`.
-   **File:** `sdtm-standards/src/ct_loader.rs`
-   **Changes:**
    -   Change `load_default_ct_registry` to return `Result<Arc<TerminologyRegistry>>`.
    -   The `OnceLock` should store `Arc<TerminologyRegistry>`.
    -   Remove `.clone()` calls when returning the registry.

### 3.2. Pipeline Context (`sdtm-core`)
-   **Task:** Update `PipelineContext` to hold `Arc`.
-   **File:** `sdtm-core/src/pipeline_context.rs`
-   **Changes:**
    -   `pub ct_registry: Arc<TerminologyRegistry>`
    -   `pub standards: Arc<SdtmStandards>` (if not already)

### 3.3. String Interning (`sdtm-model`)
-   **Task:** Reduce heap overhead for millions of small strings.
-   **File:** `sdtm-model/src/ct.rs` (or wherever `Term` is defined).
-   **Changes:**
    -   Add `smol_str = "0.2"` to `Cargo.toml`.
    -   Replace `String` with `SmolStr` for:
        -   `Term.code` (always short)
        -   `Term.submission_value` (often short)
        -   `Codelist.submission_value`

## Phase 4: API Surface Reduction
**Goal:** Enforce boundaries to prevent "leaky" abstractions.

### 4.1. Privatize `sdtm-model`
-   **Task:** Audit `pub` structs.
-   **Action:**
    -   Identify structs that are only used to pass data between `sdtm-ingest` and `sdtm-core`.
    -   Change visibility to `pub(crate)` or move them to a `common` internal crate if they don't need to be exposed to the CLI user or external consumers.

### 4.2. Simplify `sdtm-ingest` API
-   **Task:** Hide helper functions.
-   **Action:**
    -   Make `normalize_header`, `normalize_cell`, `any_to_string` private or `pub(crate)`.
    -   Users of the crate should only see `read_csv_table` and `discover_domain_files`.

## Phase 5: Dependency Cleanup
**Goal:** Remove unused crates to speed up compile time and reduce binary size.

### 5.1. Remove `csv` Crate
-   **Task:** After Phase 1, check `sdtm-ingest/Cargo.toml`.
-   **Action:** Remove `csv` dependency if Polars is handling all I/O.

### 5.2. Check `sdtm-transform`
-   **Task:** Verify if `sdtm-transform` is still needed.
-   **Action:** If its logic is moved to `sdtm-core` vectorization, merge it into `sdtm-core` or delete it.

## Execution Order & Verification

1.  **Step 1 (Phase 1):** Refactor `sdtm-ingest`.
    *   *Verify:* Run `cargo test -p sdtm-ingest`. Expect breakages in `sdtm-core`.
2.  **Step 2 (Phase 1):** Fix `sdtm-core` compilation errors.
    *   *Verify:* `cargo check`.
3.  **Step 3 (Phase 2):** Implement Vectorization in `sdtm-core`.
    *   *Verify:* Run `cargo test -p sdtm-core`.
4.  **Step 4 (Phase 3):** Apply `Arc` and `SmolStr`.
    *   *Verify:* `cargo bench` (if benchmarks exist) or time the CLI execution.
5.  **Step 5 (Phase 4/5):** Cleanup and Privatize.
    *   *Verify:* `cargo doc --open` to see the reduced public API.
