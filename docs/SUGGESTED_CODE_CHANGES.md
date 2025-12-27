# SDTM Transpiler Implementation Task List

This task list is a strict, ordered implementation guide for the SDTM
Transpiler. Tasks are organized into phases with sequential task numbers. **An
agent must execute tasks in order within each phase.** Cross-phase dependencies
are noted where applicable.

## Implementation Rules

1. **Execute tasks in numerical order** within each phase.
2. **Do not skip tasks** unless explicitly marked as completed `[x]`.
3. **Mark tasks completed** immediately after finishing (`[x]`).
4. **Run validation** after each task:
   `cargo fmt && cargo clippy && cargo test`.
5. **Update this document** if requirements change during implementation.

## SDTM Standards Policy

> **CRITICAL: Never fabricate or assume SDTM rules.**

1. **Always verify** SDTM requirements against the official documentation in
   `standards/sdtmig/v3_4/chapters/` before implementing any rule or validation.
2. **Reference chapters** contain the full SDTMIG v3.4 specification in
   Markdown:
   - `chapter_02_fundamentals-of-the-sdtm.md` - Core concepts
   - `chapter_03_submitting-data-in-standard-format.md` - Submission metadata
   - `chapter_04_assumptions-for-domain-models.md` - Variable rules, timing,
     text
   - `chapter_05_models-for-special-purpose-domains.md` - DM, SE, SV, CO
   - `chapter_06_domain-models-based-on-the-general-observation-classes.md` -
     Findings, Events, Interventions
   - `chapter_07_trial-design-model-datasets.md` - TA, TE, TV, TI, TS, TD, TM
   - `chapter_08_representing-relationships-and-data.md` - RELREC, RELSPEC,
     RELSUB, SUPPQUAL
   - `chapter_09_study-references.md` - DI, OI
   - `chapter_10_appendices.md` - CT, naming fragments, QNAM codes
3. **When in doubt**, read the relevant chapter section before coding.
4. **Cite chapter/section** in code comments when implementing SDTMIG rules.
5. **Do not invent** variable constraints, CT values, or domain rules that are
   not explicitly documented in the standards.

---

## Reference: SDTMIG v3.4 Domain Coverage

- **Source of truth**: `standards/sdtmig/v3_4/Datasets.csv` and
  `standards/sdtmig/v3_4/Variables.csv`.
- **Domains in SDTMIG v3.4 (63)**: AE, AG, BE, BS, CE, CM, CO, CP, CV, DA, DD,
  DM, DS, DV, EC, EG, EX, FA, FT, GF, HO, IE, IS, LB, MB, MH, MI, MK, ML, MS,
  NV, OE, OI, PC, PE, PP, PR, QS, RE, RELREC, RELSPEC, RELSUB, RP, RS, SC, SE,
  SM, SR, SS, SU, SUPPQUAL, SV, TA, TD, TE, TI, TM, TR, TS, TU, TV, UR, VS.
- **Current Rust domain processors (17)**: AE, CM, DA, DM, DS, EX, IE, LB, MH,
  PE, PR, QS, SE, TA, TE, TS, VS. All other domains use default processing.
- **Relationship datasets**: RELREC, RELSPEC, RELSUB, SUPPQUAL.

---

# Phase 0: Foundations and Infrastructure

> **Goal**: Establish clean architecture, shared utilities, observability, and
> strict processing policies. All subsequent phases depend on Phase 0.

## 0.1 Shared Utilities Consolidation

- [x] **0.1.1** Consolidate `any_to_string` and numeric parsing logic into a
      single utility module in `crates/sdtm-core/src/data_utils.rs`. Remove
      duplicates from `domain_processors/common.rs`, `relationships.rs`, and
      `sdtm-report/src/lib.rs`.

- [x] **0.1.2** Add a case-insensitive column lookup helper in `sdtm-model` or
      `sdtm-core`. Use it across `processor.rs`, `preprocess.rs`, and
      `validate/lib.rs`.

- [x] **0.1.3** Build a `reference_starts` map from DM (RFSTDTC) and pass it
      into `ProcessingContext` for SDY derivation.

- [ ] **0.1.4** Create a shared date/time module in `sdtm-core` with ISO 8601
      parsing, validation, and normalization. Remove duplicate date logic from
      domain processors.

- [ ] **0.1.5** Standardize CT normalization via a single API in
      `sdtm-core/src/ct_utils.rs` with strict and explicit modes. Remove
      inconsistent CT resolution algorithms.

## 0.2 Architecture and Pipeline Refactor

- [ ] **0.2.1** Introduce `StudyPipelineContext` struct to cache standards, CT
      registry, P21 rules, and metadata. Pass it through all pipeline stages.

- [ ] **0.2.2** Extend `DomainFrame` with `DomainFrameMeta` containing:
      `dataset_name`, `source_files`, `split_variant`, `base_domain_code`.
      Propagate into outputs and reports.

- [ ] **0.2.3** Refactor `run_study` in `sdtm-cli/src/commands.rs` into explicit
      pipeline stages:
      `ingest -> map -> preprocess -> domain_rules ->
      validate -> output`.
      Use typed input/output structs between stages.

- [ ] **0.2.4** Replace hard-coded domain processor match in
      `domain_processors/mod.rs` with a registry map
      (`HashMap<String,
      Box<dyn DomainProcessor>>`). Allow processors to
      be added/disabled via config.

- [ ] **0.2.5** Define ordered per-domain step lists and run them through a
      single pipeline executor. Remove nested helper chains.

## 0.3 Standards Ingestion and Rule Registry

- [ ] **0.3.1** Extend `sdtm-standards/src/loaders.rs` to capture dataset Class
      from `Datasets.csv`. Map rules by class (General Observation,
      Special-Purpose, Trial Design, Study Reference).

- [ ] **0.3.2** Create a curated `sdtmig_assumptions` registry (YAML or TOML)
      per domain with explicit rules and chapter/page citations from
      `standards/sdtmig/v3_4/chapters`.

- [ ] **0.3.3** Build a rule engine in `sdtm-validate` that runs assumption
      rules in strict mode and returns structured issues (no string parsing).

- [ ] **0.3.4** Store CT release version and publishing set in the standards
      registry. Emit in Define-XML with strict CT validation.

## 0.4 Observability and Logging

- [ ] **0.4.1** Adopt `tracing` with a shared `tracing-subscriber` init in
      `sdtm-cli/src/logging.rs`. Route all logs through structured spans.

- [ ] **0.4.2** Instrument pipeline stages (ingest, mapping, preprocess,
      validation, output) with spans that log row counts, domain counts, and
      durations.

- [ ] **0.4.3** Include `study_id`, `domain_code`, `dataset_name`, and source
      file path in log fields. Propagate context through the pipeline.

- [ ] **0.4.4** Add CLI flags: `--verbose`, `--quiet`, `--log-level`,
      `--log-format`, `--log-file`. Ensure consistent behavior across
      subcommands.

- [ ] **0.4.5** Default to redacted logging for row-level data. Require explicit
      `--log-data` flag for PHI values. Document PHI-safe logging rules in
      README.

## 0.5 Strictness and Non-Fabrication Policy

- [ ] **0.5.1** Make strict mode mandatory. Block outputs when strict validation
      fails. Remove lenient mode fallbacks.

- [ ] **0.5.2** Remove or gate any auto-fill logic not explicitly sourced from
      input data, study metadata, or SDTMIG-approved derivations. Document
      approved derivation rules.

- [ ] **0.5.3** Restrict CT normalization to exact or synonym mappings only.
      Require explicit mapping metadata for fuzzy matches.

- [ ] **0.5.4** Require explicit mapping/derivation config for `USUBJID`. Never
      drop rows silently. Fail strict validation on missing required
      identifiers.

- [ ] **0.5.5** In strict mode, only apply value normalization (e.g., DM RACE,
      SEX, AGEU) when a mapping file explicitly allows it. Otherwise error and
      preserve raw values.

- [ ] **0.5.6** Record provenance for every derived value (SDY, sequence, CT
      normalization). Expose origin in reports and Define-XML.

- [ ] **0.5.7** Add validation rule that flags any field populated without a
      documented source or derivation rule (no-imputation check).

## 0.6 Ingest and Mapping Improvements

- [ ] **0.6.1** Allow explicit header row index or schema hints to override
      heuristics in `sdtm-ingest/src/csv_table.rs`.

- [ ] **0.6.2** Store mapping configs and reuse them across runs via a mapping
      repository (like Python legacy).

- [ ] **0.6.3** Incorporate variable labels, synonyms, and domain patterns from
      standards into the mapping engine in `sdtm-map/src/engine.rs`.

- [ ] **0.6.4** Use Polars CSV streaming for large datasets. Only sample rows
      for hints.

## 0.7 Domain Processing Refactor

- [ ] **0.7.1** Split `sdtm-core/src/preprocess.rs` into per-domain modules
      (e.g., `preprocess/dm.rs`, `preprocess/ae.rs`). Drive with a rule table.

- [ ] **0.7.2** Remove or require explicit mapping metadata before populating
      any value in preprocess/domain processors.

- [ ] **0.7.3** Use Polars expressions for bulk transforms
      (trimming/uppercasing) instead of per-row loops.

## 0.8 Validation Refactor

- [ ] **0.8.1** Add cross-domain validations: split SEQ uniqueness, SUPP QNAM
      uniqueness, QVAL non-empty, relationship key integrity.

- [ ] **0.8.2** Add explicit rule metadata mapping in
      `sdtm-standards/src/loaders.rs`. Use it in `sdtm-validate/src/lib.rs` for
      missing dataset detection.

- [ ] **0.8.3** Add a structured issue model with counts and samples. Update
      `sdtm-cli/src/summary.rs` to render it.

---

# Phase 1: SDTMIG Conformance and Rules

> **Goal**: Implement SDTMIG v3.4 rules, validations, and domain-specific
> assumptions. Depends on Phase 0 completion (especially 0.3, 0.5, 0.8).

## 1.1 Temporal and ISO 8601 Rules (Chapter 4)

- [ ] **1.1.1** Implement strict ISO 8601 parser/validator in the shared date
      module for `--DTC`, `--STDTC`, `--ENDTC`, and `--DUR`. Support extended
      format, partials, and intervals. Error on basic format.

- [ ] **1.1.2** Stop mutating end dates in `ensure_date_pair_order`. Emit hard
      validation error when end < start.

- [ ] **1.1.3** Compute `--DY` only when full dates are available. Document
      standard formula relative to RFSTDTC. Flag partial dates when `--DY` is
      present.

- [ ] **1.1.4** Enforce Findings date/time rules: require `--DTC` for collection
      timing, disallow `--STDTC` in Findings, allow `--ENDTC` only for interval
      collections (Section 4.4.8).

- [ ] **1.1.5** Enforce relative timing variables: validate `--STRF/--ENRF` and
      `--STRTPT/--ENRTPT` allowed values, require `--STTPT/--ENTPT` anchors,
      avoid derived `--STRF/--ENRF` when dates are collected.

- [ ] **1.1.6** Validate ISO 8601 `--DUR` values. Only allow `--DUR` when
      start/end dates are not collected. Document collected vs derived
      durations.

- [ ] **1.1.7** Derive EPOCH from `--DTC` (Findings) or `--STDTC`
      (Events/Interventions) using TA/SE reference periods. Never impute. Leave
      null for pre-study records.

## 1.2 Identifier and Sequence Rules

- [ ] **1.2.1** Enforce `--SEQ` uniqueness across split datasets. Check
      collisions across input files. Either renumber with tracking or emit hard
      error.

- [ ] **1.2.2** Validate General Observation identifiers: `STUDYID`, `DOMAIN`,
      `USUBJID`, and `--SEQ` for GO domains (except DM) across all records.

- [ ] **1.2.3** Validate variable prefixes follow base `DOMAIN` code, not
      dataset name, for split datasets.

- [ ] **1.2.4** Propagate `variant` from `sdtm-ingest/src/discovery.rs` into
      outputs and reports for split dataset names.

## 1.3 Length and Text Rules

- [ ] **1.3.1** Validate SDTMIG/SAS V5 length limits: `--TEST` <= 40 (except
      IE/TI/TS), `--TESTCD`/`QNAM` <= 8, `QLABEL` <= 40.

- [ ] **1.3.2** Implement long-text splitting (>200 chars) into parent +
      `QNAM1`, `QNAM2` SUPP records with word-boundary splitting and QLABEL
      rules (Section 4.5.3).

- [ ] **1.3.3** Enforce QNAM/QLABEL constraints: QNAM <= 8, QLABEL <= 40, QVAL
      non-empty, uniqueness across
      `STUDYID,RDOMAIN,USUBJID,IDVAR,IDVARVAL,QNAM`.

## 1.4 SUPPQUAL and Relationship Rules (Chapter 8)

- [ ] **1.4.1** Generate SUPP names from dataset name with SQ fallback when >8
      chars (Section 8.4.2).

- [ ] **1.4.2** Set blank IDVAR/IDVARVAL for DM SUPP records. Validate
      accordingly.

- [ ] **1.4.3** Generate RELREC only from explicit relationship keys. Add
      configuration to disable auto RELREC.

- [ ] **1.4.4** Set RELTYPE only for dataset-level relationships (Section 8.3).
      Leave blank for record-level links.

- [ ] **1.4.5** Implement and validate RELSPEC (Related Specimens) per Chapter
      8.

- [ ] **1.4.6** Implement and validate RELSUB (Related Subjects) per Chapter 8.

- [ ] **1.4.7** Implement CO IDVAR/IDVARVAL rules. Avoid generating RELREC when
      CO already links records.

- [ ] **1.4.8** Require expected key columns for RELREC/RELSPEC/RELSUB before
      emitting records.

- [ ] **1.4.9** Validate uniqueness across splits when IDVAR is not --SEQ
      (Section 4.1.7 rule 4).

- [ ] **1.4.10** Block timing variables in SUPPQUAL and RELREC unless explicitly
      allowed by SDTMIG.

## 1.5 Variable Order and Core Rules

- [ ] **1.5.1** Order dataset columns by SDTM role: Identifiers, Topic,
      Qualifiers, Timing. Validate output order.

- [ ] **1.5.2** Order Define-XML ItemRefs by SDTM role order.

- [ ] **1.5.3** Enforce Core designation rules: require all Required columns
      with non-null values, include Expected columns even when uncollected with
      Define-XML comment, include Permissible only when collected.

## 1.6 Domain Naming and Custom Codes

- [ ] **1.6.1** Enforce dataset name = DOMAIN code (except split datasets).
      Validate 2-character domain codes.

- [ ] **1.6.2** Disallow AD/AX/AP/SQ/SA for custom domains. Prefer X/Y/Z
      prefixes for custom codes.

- [ ] **1.6.3** Reject variables from other observation classes in standard
      domains. Move sponsor variables to SUPPQUAL (no renaming/repurposing).

- [ ] **1.6.4** Add denylist validator for SEND-only variables (--USCHFL,
      --METHOD, --RSTIND, --IMPLBL, RP* timing vars, --NOMDY, --DETECT) and DM
      non-host variables (SPECIES, STRAIN, SBSTRAIN, RPATHCD).

- [ ] **1.6.5** Add dependency checks: SETCD requires Trial Sets, POOLID
      requires Pool Definition.

## 1.7 Submission Metadata and Conformance (Chapter 3)

- [ ] **1.7.1** Populate Define-XML dataset metadata (Class, Structure, Purpose,
      Keys, Location) from standards + overrides. Validate against actual data.

- [ ] **1.7.2** Compute natural keys from standards. Ensure record uniqueness.
      Surface key violations. Align Define-XML Keys with actual uniqueness.

- [ ] **1.7.3** Include Expected columns as all-null with Define-XML comment
      "not collected". Omit Permissible columns when not collected.

- [ ] **1.7.4** Generate value-level rules from test code/category. Emit
      `ValueListDef`/`WhereClauseDef` with per-test attributes.

- [ ] **1.7.5** Implement strict conformance validator for standard names,
      types, CT usage, required identifiers, and timing variables (Section
      3.2.2).

- [ ] **1.7.6** Only generate domains with collected/derived data unless
      explicitly configured. Require Define-XML comment when empty domain
      included.

## 1.8 Special-Purpose Domain Rules (Chapter 5)

- [ ] **1.8.1** Implement CO long comment splitting: `COVAL`, `COVAL1...COVALn`
      200-char splitting with ordered segments and word-boundary splitting.

- [ ] **1.8.2** Enforce CO linking rules: RDOMAIN/IDVAR/IDVARVAL rules, CODTC
      null when related to parent records, RDOMAIN/IDVAR/IDVARVAL null for
      general comments.

- [ ] **1.8.3** Block CO non-standard qualifiers: `--GRPID`, `--REFID`,
      `--SPID`, `TAETORD`, `--TPT*`, `--RFTDTC`.

- [ ] **1.8.4** Validate DM ARM/ARMCD/ACTARM alignment to TA: 1:1 ARM/ARMCD
      mapping, ARMNRS rules, multistage truncation logic. Require ACTARMUD when
      ARMNRS="UNPLANNED TREATMENT".

- [ ] **1.8.5** Disallow DM population flags (COMPLT/FULLSET/ITT/PPROT/SAFETY).
      Require these in ADaM.

- [ ] **1.8.6** Enforce multiple race handling: set RACE="MULTIPLE" when
      multiple races collected, store additional races in SUPPDM with standard
      QNAMs, enforce CT for RACE/ETHNIC.

- [ ] **1.8.7** Restrict DM to allowed variable set (VISIT/VISITNUM/VISITDY and
      DMXFN only as extras). Error on unexpected columns.

- [ ] **1.8.8** Enforce SE chronology: SESEQ chronological order by SESTDTC,
      ETCD in TE unless UNPLAN, SEUPDES when ETCD=UNPLAN, TAETORD null for
      unplanned/out-of-place elements.

- [ ] **1.8.9** Add Define-XML comments for SESTDTC/SEENDTC derivation rules
      when inferred from TE/EX/DS.

- [ ] **1.8.10** Enforce SV planned/unplanned rules: 1 SV per VISITNUM per
      subject, SVPRESP/SVOCCUR/SVREASOC logic, align planned SV visits to TV.

## 1.9 General Observation Class Rules (Chapter 6)

- [ ] **1.9.1** Enforce Findings results standardization: capture
      `--ORRES/--ORRESU`, derive `--STRESC/--STRESN/--STRESU`, populate numeric
      only when conversion valid.

- [ ] **1.9.2** Validate `--TESTCD`/`--TEST` against CT: uppercase `--TESTCD` <=
      8, `--TEST` <= 40, require CT or sponsor codelist, reject nonstandard in
      strict mode.

- [ ] **1.9.3** Enforce dictionary coding: `--MODIFY` and `--DECOD` usage per
      domain, PESTRESC for PE coding, record dictionary metadata in Define-XML.

- [ ] **1.9.4** Enforce `--PRESP/--OCCUR/--STAT/--REASND` rules: prespecified vs
      occurrence logic, require `--STAT="NOT DONE"` to have `--REASND`, disallow
      contradictory values.

- [ ] **1.9.5** Calculate baseline/LOBXFL using RFXSTDTC. Ensure single last
      pre-treatment record per test per subject.

- [ ] **1.9.6** Validate domain pairs and FA relationships: MB/MS and PC/PP
      parent-child linking, RELREC dataset-level relationships, FA dataset
      naming when split by parent.

## 1.10 Trial Design Model Alignment (Chapter 7)

- [ ] **1.10.1** Validate TAETORD/ETCD/EPOCH constraints: TAETORD integer
      ordering within ARMCD, ETCD length <= 8, EPOCH consistency across arms.

- [ ] **1.10.2** Enforce TA/TE/SE/DM alignment: TA ETCD must exist in TE, DM
      ARM/ARMCD matches TA, SE ETCD/EPOCH and ACTARMCD derivation aligns with
      TA/TE.

- [ ] **1.10.3** Require TV when VISIT/VISITNUM appear in subject domains.
      Validate SV planned visits against TV.

- [ ] **1.10.4** Enforce TI/IE linkage: IE records reference TI IETESTCD/IETEST
      values, require TI definitions for all IE criteria.

- [ ] **1.10.5** Implement Trial Summary (TS), Trial Disease Assessments (TD),
      and Trial Disease Milestones (TM) with required parameters and unique
      keys.

## 1.11 Study Reference Datasets (Chapter 9)

- [ ] **1.11.1** Implement Device Identifiers (DI) per SDTMIG-MD. Require
      SPDEVID values in subject domains to exist in DI.

- [ ] **1.11.2** Implement Non-host Organism Identifiers (OI): NHOID uniqueness,
      OIPARMCD/OIPARM/OIVAL completeness, hierarchical OISEQ order, forbid extra
      variables.

## 1.12 Appendix-Driven Naming and CT Checks (Chapter 10)

- [ ] **1.12.1** Validate QNAMs against Appendix C1. Mark nonstandard QNAMs with
      `def:IsNonStandard`.

- [ ] **1.12.2** Implement fragment-based generator/hint system (Appendix D) for
      `--TESTCD`/QNAM to keep codes <= 8 chars and avoid collisions.

---

# Phase 2: Output Generation and MSG Parity

> **Goal**: Achieve MSG v2.0 sample parity for Define-XML, Dataset-XML, and XPT
> outputs. Depends on Phase 0 (especially 0.2.2 for DomainFrameMeta) and Phase 1
> conformance rules.

## 2.1 Output Infrastructure

- [ ] **2.1.1** Pass `CtRegistry` into `write_define_xml` and
      `write_dataset_xml`. Remove internal CT registry reload.

- [ ] **2.1.2** Use `DomainFrameMeta.dataset_name` for XPT/XML file names and
      metadata. Ensure split dataset names are preserved.

- [ ] **2.1.3** Stream dataset XML writing directly from frames instead of
      building in memory.

## 2.2 Define-XML MSG v2.0 Parity

- [ ] **2.2.1** Emit `<?xml-stylesheet?>` PI referencing `define2-1.xsl`.

- [ ] **2.2.2** Emit `<def:Standards>` section with IG/CT standards (Name, Type,
      Version, Status, PublishingSet from SDTMIG/CT versions).

- [ ] **2.2.3** Add `<def:AnnotatedCRF>` and `<def:SupplementalDoc>` with
      `<def:DocumentRef>` entries when aCRF/cSDRG supplied.

- [ ] **2.2.4** Emit `<def:leaf>` for each dataset/document. Link via
      `def:ArchiveLocationID` on `ItemGroupDef`.

- [ ] **2.2.5** Populate ItemGroupDef: `Purpose`, `def:StandardOID`,
      `<def:Class Name=...>`, set `Repeating` based on domain structure.

- [ ] **2.2.6** Populate ItemRef: `Role` from `Variables.csv`, `MethodOID` when
      derivation present.

- [ ] **2.2.7** Add ItemDef `SASFieldName`. Choose `integer` vs `float` based on
      data characteristics.

- [ ] **2.2.8** Emit `<def:Origin>` with Type/Source and `<def:DocumentRef>` for
      CRF-based items.

- [ ] **2.2.9** Implement `<def:ValueListDef>` and `<def:WhereClauseDef>` for
      value-level items (ORRES/ORRESU, SUPP QNAM/QVAL) using value-level rules
      table.

- [ ] **2.2.10** Add CodeList `Alias` (NCI codes) and `def:IsNonStandard` where
      appropriate.

- [ ] **2.2.11** Emit `<ExternalCodeList>` for MedDRA/SNOMED/ISO when used.

- [ ] **2.2.12** Point leaf links to Dataset-XML or XPT depending on configured
      outputs.

## 2.3 Dataset-XML MSG v2.0 Parity

- [ ] **2.3.1** Set root attributes: `data:DatasetXMLVersion="1.0.0"`,
      `SourceSystem`, `SourceSystemVersion` from config.

- [ ] **2.3.2** Use FileOID format:
      `{DefineFileOID}(IG.{base_domain}).Data({dataset_name})`. Set
      `PriorFileOID` to define.xml.

- [ ] **2.3.3** For split datasets (e.g., LBCH) emit `ItemGroupOID="IG.LB"`
      while using split dataset name in `Data(...)` and file name.

- [ ] **2.3.4** Choose ReferenceData vs ClinicalData container based on domain
      class from standards.

- [ ] **2.3.5** Add optional MSG-style header comments (dataset name + source)
      without affecting XML validity.

- [ ] **2.3.6** Order ItemData by Variable Order from `Variables.csv`.

- [ ] **2.3.7** Format numeric strings as plain decimals (no scientific
      notation).

- [ ] **2.3.8** Write split datasets to `dataset-xml/split/` directory (mirror
      MSG layout).

## 2.4 XPT (SAS V5 Transport) MSG v2.0 Parity

- [ ] **2.4.1** Plumb header metadata (SAS version, OS name, created/modified)
      from config into `XptWriterOptions`. Assert deterministic values for
      strict outputs.

- [ ] **2.4.2** Carry dataset_name through `DomainFrameMeta`. Use in
      `build_xpt_dataset` and file naming. Keep base domain in metadata only.

- [ ] **2.4.3** Write label only when explicitly defined in standards or input
      metadata. Otherwise leave XPT label blank (spaces) for MSG parity.

- [ ] **2.4.4** In strict mode, reject non-ASCII values and surface validation
      error instead of replacing with `?` in `pad_ascii`.

- [ ] **2.4.5** Compute column lengths once (from standards/config) and reuse
      for XPT, Define-XML, and Dataset-XML.

- [ ] **2.4.6** Validate SDTMIG length limits (8-char identifiers, 40-char
      labels, 200-char text) and fail fast before writing XPT.

- [ ] **2.4.7** Force all numeric columns to length 8. Ignore smaller overrides.

- [ ] **2.4.8** Add fixed decimal renderer for numeric-to-char conversions (no
      scientific notation).

- [ ] **2.4.9** Add validation step comparing XPT vs Define-XML: dataset name,
      column order, labels, lengths.

- [ ] **2.4.10** Write split datasets to `xpt/split/` directory (e.g.,
      `xpt/split/lbch.xpt`).

---

# Phase 3: Performance, Testing, and Quality

> **Goal**: Optimize performance, add comprehensive tests, and ensure parity
> with MSG samples. Can run in parallel with late Phase 2 tasks.

## 3.1 Performance Optimizations

- [ ] **3.1.1** Replace `BTreeMap`/`BTreeSet` with `HashMap`/`HashSet` in hot
      code paths (`processor.rs`, `engine.rs`) where order is not required.

- [ ] **3.1.2** Cache normalized (uppercase) keys and reuse across pipeline
      steps. Avoid repeated `to_uppercase` allocations.

- [ ] **3.1.3** Use mutable references and in-place transforms in
      `sdtm-cli/src/commands.rs`. Avoid unnecessary DataFrame clones.

## 3.2 Test Coverage

- [ ] **3.2.1** Add tests for split dataset naming parity (QS36/FACM) across all
      outputs and SUPP datasets.

- [ ] **3.2.2** Add tests for long-text SUPP splitting per SDTMIG Section 4.5.3
      (>200 char).

- [ ] **3.2.3** Add parity tests for RELREC RELTYPE usage (record-level vs
      dataset-level).

- [ ] **3.2.4** Add tests for SDY computation via DM `reference_starts`.

- [ ] **3.2.5** Add parity tests against MSG sample outputs for Define-XML,
      Dataset-XML, and XPT.

---

# Phase 4: Python Removal and Cutover

> **Goal**: Complete migration from Python to Rust-only. Execute only after
> Phases 0-3 are complete and parity is confirmed.

## 4.1 Parity Verification

- [ ] **4.1.1** Build a parity checklist against `cdisc_transpiler/` Python
      codebase. Document Rust coverage for each feature.

- [ ] **4.1.2** Run parallel outputs (Python vs Rust) on test datasets. Document
      any differences.

## 4.2 Python Removal

- [ ] **4.2.1** Tag a final Python release. Archive to `legacy/` repo or release
      asset.

- [ ] **4.2.2** Remove `cdisc_transpiler/` directory.

- [ ] **4.2.3** Delete Python entry points and packaging files
      (`pyproject.toml`, `requirements.txt`, `cdisc_transpiler.egg-info`).

- [ ] **4.2.4** Delete any Python interop hooks, config flags, or tests that
      depend on Python output.

## 4.3 Documentation and CI Cleanup

- [ ] **4.3.1** Remove Python CI jobs from workflow files.

- [ ] **4.3.2** Update `README.md` to Rust-only documentation. Remove Python
      references.

- [ ] **4.3.3** Document migration path from Python to Rust CLI.

- [ ] **4.3.4** Communicate Python deprecation to users.

---

# Appendix: Task Dependencies

| Task   | Depends On                                    |
| ------ | --------------------------------------------- |
| 0.2.*  | 0.1.* (shared utilities)                      |
| 0.5.*  | 0.3.* (rule registry), 0.4.* (logging)        |
| 0.7.*  | 0.1.4 (date module), 0.1.5 (CT API)           |
| 0.8.*  | 0.3.* (rule registry), 0.2.1 (context)        |
| 1.1.*  | 0.1.4 (date module)                           |
| 1.2.*  | 0.2.2 (DomainFrameMeta)                       |
| 1.4.*  | 0.8.1 (cross-domain validation)               |
| 1.5.*  | 0.3.1 (dataset Class)                         |
| 1.7.*  | 0.3.* (rule registry), 1.5.* (variable order) |
| 1.8.*  | 1.4.* (SUPPQUAL rules)                        |
| 1.10.* | 1.8.4 (DM alignment)                          |
| 2.1.*  | 0.2.2 (DomainFrameMeta)                       |
| 2.2.*  | 1.5.* (variable order), 1.7.* (metadata)      |
| 2.3.*  | 2.1.* (output infra), 1.5.1 (column order)    |
| 2.4.*  | 2.1.* (output infra), 1.3.1 (length limits)   |
| 3.2.*  | 2.* (outputs complete)                        |
| 4.*    | 0._, 1._, 2._, 3._ (all phases complete)      |

---

# Progress Summary

| Phase     | Total   | Completed | Remaining |
| --------- | ------- | --------- | --------- |
| 0         | 35      | 3         | 32        |
| 1         | 67      | 0         | 67        |
| 2         | 32      | 0         | 32        |
| 3         | 8       | 0         | 8         |
| 4         | 8       | 0         | 8         |
| **Total** | **150** | **3**     | **147**   |
