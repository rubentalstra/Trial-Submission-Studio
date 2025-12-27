# SDTM Transpiler Refactor and Optimization Task List

This task list is based on a review of the Rust crates and SDTMIG v3.4 guidance
in `standards/sdtmig/v3_4/chapters` (Ch 2-10). Goal: keep the pipeline
clean, lean, and dynamic; reduce nested helper chains; remove legacy Python once
parity is reached.

## Domain coverage inventory (SDTMIG v3.4)
- Source of truth: `standards/sdtmig/v3_4/Datasets.csv` and `standards/sdtmig/v3_4/Variables.csv`.
- Domains listed in SDTMIG v3.4 (63): AE, AG, BE, BS, CE, CM, CO, CP, CV, DA, DD, DM, DS, DV, EC, EG, EX, FA, FT, GF, HO, IE, IS, LB, MB, MH, MI, MK, ML, MS, NV, OE, OI, PC, PE, PP, PR, QS, RE, RELREC, RELSPEC, RELSUB, RP, RS, SC, SE, SM, SR, SS, SU, SUPPQUAL, SV, TA, TD, TE, TI, TM, TR, TS, TU, TV, UR, VS.
- Current Rust domain processors implemented (17): AE, CM, DA, DM, DS, EX, IE, LB, MH, PE, PR, QS, SE, TA, TE, TS, VS. All other domains currently rely on default processing only.
- Relationship datasets present in standards: RELREC, RELSPEC, RELSUB, SUPPQUAL (needs strict SDTMIG behavior and validations).
- [ ] Problem: missing visibility on which domains are fully compliant. Fix: add a per-domain gap matrix (processing, validation, outputs, MSG parity) and keep it updated in this doc.

## Phase plan for full SDTMIG compliance
- [ ] Problem: no agreed rollout sequence for full SDTMIG coverage. Fix: publish a domain-by-domain gap report (processing + validations + outputs) using SDTMIG CSVs + chapter assumptions, and rank by priority.
- [ ] Problem: strict SDTMIG assumptions are only partially implemented. Fix: implement strict assumptions and validations for all domains in phases, starting with DM/AE/LB/VS/EX/DS and Trial Design (TA/TE/TS/TI/TV).
- [ ] Problem: relationship datasets are not uniformly handled. Fix: complete RELREC/RELSPEC/RELSUB/SUPP/CO behavior and validations in Phase 3 with explicit-only rules.

## Standards ingestion and rule registry
- [ ] Problem: SDTMIG assumptions live only in narrative chapters. Fix: create a curated `sdtmig_assumptions` registry (YAML/TOML) per domain with explicit rules and chapter/page citations from `standards/sdtmig/v3_4/chapters`.
- [ ] Problem: no structured assumption engine. Fix: build a rule engine in `crates/sdtm-validate` that runs assumption rules in strict mode and returns structured issues (no string parsing).
- [ ] Problem: dataset Class is not used to drive rules. Fix: extend `crates/sdtm-standards/src/loaders.rs` to capture dataset Class and map rules by class (General Observation vs Special-Purpose vs Trial Design vs Study Reference).

## Strictness and non-fabrication policy
- [ ] Problem: pipeline can run in a lenient mode. Fix: make strict mode mandatory and block outputs when strict validation fails.
- [ ] Problem: heuristic fill logic can invent values. Fix: remove or gate any auto-fill that is not explicitly sourced from input data, study metadata, or SDTMIG-approved derivations.
- [ ] Problem: derived values lack provenance. Fix: record provenance for every derived value (e.g., SDY, sequence assignment, CT normalization) and expose origin in reports/Define-XML.
- [ ] Problem: fuzzy CT matching can invent mappings. Fix: restrict CT normalization to exact or synonym mappings and require explicit mapping metadata when used.
- [ ] Problem: no explicit no-imputation check. Fix: add a validation rule that flags any field populated without a documented source or derivation rule.
- [ ] Problem: `drop_placeholder_rows` auto-derives `USUBJID` and drops rows with missing `USUBJID`. Fix: require explicit mapping/derivation config for `USUBJID`, never drop rows silently, and fail strict validation on missing required identifiers.
- [ ] Problem: domain processors normalize values (e.g., DM `RACE`, `SEX`, `AGEU`) without explicit mapping. Fix: in strict mode, only apply normalization when a mapping file explicitly allows it, otherwise error and preserve raw values.

## Observability and logging
- [ ] Problem: logging is ad-hoc and inconsistent across crates. Fix: adopt `tracing` with a shared `tracing-subscriber` init in the CLI and route all logs through structured spans.
- [ ] Problem: pipeline progress is opaque. Fix: instrument ingest/mapping/preprocess/validation/output with spans that log row counts, domain counts, and durations per stage.
- [ ] Problem: logs lack context for troubleshooting. Fix: include `study_id`, `domain_code`, `dataset_name`, and source file path in log fields and propagate context through the pipeline.
- [ ] Problem: no configurable verbosity. Fix: add CLI flags (`--verbose`, `--quiet`, `--log-level`, `--log-format`, `--log-file`) with sane defaults and consistent behavior across subcommands.
- [ ] Problem: logs may leak sensitive values. Fix: default to redacted logging for row-level data, require an explicit `--log-data` opt-in for values, and document PHI-safe logging rules.

## Architecture and pipeline
- [ ] Problem: `run_study` is monolithic and hard to reason about. Fix: split into explicit pipeline stages (ingest, map, preprocess, domain rules, validation, outputs) with typed input/output structs in `crates/sdtm-cli/src/commands.rs`.
- [ ] Problem: standards and registries are reloaded across stages. Fix: introduce `StudyPipelineContext` to cache standards, CT registry, P21 rules, and metadata and pass it through all stages.
- [ ] Problem: dataset name and split variant are lost after ingest. Fix: add `DomainFrameMeta` (or extend `DomainFrame`) with dataset name, source files, and split variant; propagate into outputs and reports.
- [ ] Problem: domain processors are hard-coded in a match. Fix: switch to a registry map or trait objects so processors can be added/disabled without nested calls.
- [ ] Problem: domain processing uses nested helper chains. Fix: define ordered per-domain step lists and run them through a single pipeline executor.

## SDTMIG behavior alignment (from chapters)
- [ ] Problem: `--SEQ` uniqueness across split datasets is not enforced. Fix: check collisions across input files and either renumber with tracking or emit a hard error in `crates/sdtm-core/src/processor.rs`.
- [ ] Problem: General Observation identifier rules are not enforced. Fix: validate `STUDYID`, `DOMAIN`, `USUBJID`, and `--SEQ` for GO domains (except DM) across all records.
- [ ] Problem: split datasets can violate prefix rules. Fix: validate that variable prefixes follow base `DOMAIN`, not dataset name, for split datasets.
- [ ] Problem: SDTMIG/SAS V5 length limits are not enforced. Fix: validate `--TEST` <= 40 (except IE/TI/TS), `--TESTCD`/`QNAM` <= 8, `QLABEL` <= 40, and apply the long-text splitting rules from Section 4.5.3.
- [ ] Problem: split dataset names are not preserved end-to-end. Fix: propagate `variant` from `crates/sdtm-ingest/src/discovery.rs` into outputs and reports.
- [ ] Problem: SUPP dataset naming uses base domain code. Fix: generate SUPP names from dataset name with the SQ fallback when >8 chars (Section 8.4.2).
- [ ] Problem: >200 char text is not handled. Fix: split long values into parent + `QNAM1`, `QNAM2` SUPP records, with word-boundary splitting and QLABEL rules.
- [ ] Problem: QNAM/QLABEL constraints are not validated. Fix: enforce QNAM <= 8, QLABEL <= 40, QVAL non-empty, and uniqueness across `STUDYID,RDOMAIN,USUBJID,IDVAR,IDVARVAL,QNAM` in `crates/sdtm-validate/src/lib.rs`.
- [ ] Problem: RELREC can be inferred from incidental data. Fix: generate RELREC only from explicit relationship keys and allow configuration to disable auto RELREC.
- [ ] Problem: RELTYPE can be populated for record-level links. Fix: only set RELTYPE for dataset-level relationships (Section 8.3) and leave blank otherwise.
- [ ] Problem: RELSPEC/RELSUB behaviors are incomplete. Fix: implement and validate RELSPEC (Related Specimens) and RELSUB (Related Subjects) per Chapter 8.
- [ ] Problem: CO relationship handling is missing. Fix: implement IDVAR/IDVARVAL rules for CO and avoid generating RELREC when CO already links records.
- [ ] Problem: relationship datasets can be emitted without required keys. Fix: require the expected key columns for RELREC/RELSPEC/RELSUB before emitting records.
- [ ] Problem: split-domain relationships may join incorrectly with IDVAR != --SEQ. Fix: validate uniqueness across splits when IDVAR is not --SEQ (Section 4.1.7 rule 4).
- [ ] Problem: DM SUPP records require empty IDVAR/IDVARVAL. Fix: set blank IDVAR/IDVARVAL for DM SUPP records and validate accordingly.
- [ ] Problem: SDTM role-based variable order is not enforced. Fix: order dataset columns and Define-XML ItemRefs by SDTM role order (Identifiers, Topic, Qualifiers, Timing) and validate output order.
- [ ] Problem: Core designation rules (Required/Expected/Permissible) are not enforced. Fix: require all Required columns with non-null values, include Expected columns even when uncollected with a Define-XML comment, and include Permissible columns only when data were collected.
- [ ] Problem: dataset naming and custom domain code rules are not enforced. Fix: enforce dataset name = DOMAIN code (except split datasets), validate 2-character domain codes, disallow AD/AX/AP/SQ/SA for custom domains, and prefer X/Y/Z prefixes for custom codes to avoid conflicts.
- [ ] Problem: non-standard variables can appear in standard domains. Fix: reject variables from other observation classes and move sponsor variables to SUPPQUAL (no renaming/repurposing of standard variables).
- [ ] Problem: SDTMIG "variables not allowed" list is not enforced. Fix: add a denylist validator for SEND-only variables (e.g., --USCHFL, --METHOD, --RSTIND, --IMPLBL, RP* timing vars, --NOMDY, --DETECT) and DM non-host variables (SPECIES, STRAIN, SBSTRAIN, RPATHCD), and add dependency checks for SETCD (requires Trial Sets) and POOLID (requires Pool Definition).
- [ ] Problem: additional timing variables are allowed in SUPPQUAL/RELREC. Fix: block timing variables in SUPPQUAL and RELREC unless explicitly allowed by SDTMIG.

## Submission metadata and conformance (Ch 3)
- [ ] Problem: dataset-level metadata is incomplete or inconsistent. Fix: populate Define-XML dataset metadata (Class, Structure, Purpose, Keys, Location) from standards + overrides and validate against actual data.
- [ ] Problem: key structure is not validated. Fix: compute natural keys from standards and ensure record uniqueness; surface key violations and align Define-XML Keys with actual uniqueness.
- [ ] Problem: Expected variables are missing when not collected. Fix: include Expected columns as all-null with a Define-XML comment stating "not collected"; omit Permissible columns when data were not collected.
- [ ] Problem: value-level metadata rules are not linked to data. Fix: generate value-level rules from test code/category and emit `ValueListDef`/`WhereClauseDef` with per-test attributes.
- [ ] Problem: SDTMIG conformance rules are not checked. Fix: implement a strict conformance validator for standard names, types, CT usage, required identifiers, and timing variables (Section 3.2.2).
- [ ] Problem: empty or uncollected domains can be emitted. Fix: only generate domains with collected/derived data unless explicitly configured; require a Define-XML comment when an empty domain is included.

## Temporal and ISO 8601 rules (Ch 4)
- [ ] Problem: ISO 8601 validation is missing. Fix: implement a strict ISO 8601 parser/validator for `--DTC`, `--STDTC`, `--ENDTC`, and `--DUR` (extended format, no spaces, partials and intervals supported) and error on basic format.
- [ ] Problem: end dates are silently adjusted (`ensure_date_pair_order`). Fix: stop mutating end dates; emit a hard validation error when end < start.
- [ ] Problem: study day derivation ignores partial/interval dates. Fix: compute `--DY` only when full dates are available and document the standard formula relative to RFSTDTC; flag partial dates when `--DY` is present.
- [ ] Problem: Findings date/time usage is not enforced. Fix: require `--DTC` for Findings collection timing, disallow `--STDTC` in Findings, and allow `--ENDTC` only for interval collections (Section 4.4.8).
- [ ] Problem: relative timing variables accept invalid values. Fix: enforce allowed values for `--STRF/--ENRF` and `--STRTPT/--ENRTPT`, require `--STTPT/--ENTPT` anchors when used, and avoid derived `--STRF/--ENRF` when dates are collected.
- [ ] Problem: durations are not validated. Fix: validate ISO 8601 `--DUR` values, only allow `--DUR` when start/end dates are not collected, and document whether durations are collected vs derived.
- [ ] Problem: EPOCH derivation is inconsistent. Fix: derive EPOCH from `--DTC` (Findings) or `--STDTC` (Events/Interventions) using TA/SE reference periods, never impute, and leave null for pre-study records.

## Special-purpose domain assumptions (Ch 5)
- [ ] Problem: CO long comments are not split. Fix: implement `COVAL`, `COVAL1...COVALn` 200-char splitting with ordered segments and word-boundary splitting.
- [ ] Problem: CO linking rules are not enforced. Fix: enforce RDOMAIN/IDVAR/IDVARVAL rules, keep CODTC null when related to parent records, and keep RDOMAIN/IDVAR/IDVARVAL null for general comments.
- [ ] Problem: CO includes non-standard qualifiers. Fix: block `--GRPID`, `--REFID`, `--SPID`, `TAETORD`, `--TPT*`, and `--RFTDTC` in CO (per assumptions).
- [ ] Problem: DM ARM/ARMCD/ACTARM alignment to TA is not enforced. Fix: validate 1:1 ARM/ARMCD mapping, ARMNRS rules, and multistage truncation logic; require ACTARMUD when ARMNRS="UNPLANNED TREATMENT".
- [ ] Problem: DM population flags are present in SDTM. Fix: disallow COMPLT/FULLSET/ITT/PPROT/SAFETY in DM and require these to live in ADaM instead.
- [ ] Problem: multiple race handling is not enforced. Fix: set RACE="MULTIPLE" when multiple races are collected, store additional races in SUPPDM with standard QNAMs, and enforce CT for RACE/ETHNIC.
- [ ] Problem: DM allows extra variables beyond SDTMIG. Fix: restrict DM to the allowed variable set (VISIT/VISITNUM/VISITDY and DMXFN only as extras) and error on unexpected columns.
- [ ] Problem: SE chronology and UNPLAN rules are not enforced. Fix: enforce SESEQ chronological order by SESTDTC, require ETCD in TE unless UNPLAN, require SEUPDES when ETCD=UNPLAN, and leave TAETORD null for unplanned/out-of-place elements.
- [ ] Problem: SE derivations are not documented. Fix: add Define-XML comments for SESTDTC/SEENDTC derivation rules when inferred from TE/EX/DS.
- [ ] Problem: SV planned/unplanned rules are not enforced. Fix: require 1 SV per VISITNUM per subject, enforce SVPRESP/SVOCCUR/SVREASOC logic, and align planned SV visits to TV definitions.

## General observation class rules (Ch 6)
- [ ] Problem: Findings results are not standardized. Fix: enforce `--ORRES/--ORRESU` capture, derive `--STRESC/--STRESN/--STRESU`, and only populate numeric results when conversion is valid.
- [ ] Problem: `--TESTCD`/`--TEST` are not validated against CT. Fix: enforce uppercase `--TESTCD` <= 8, `--TEST` <= 40, require CT or sponsor-defined codelist, and reject nonstandard codes in strict mode.
- [ ] Problem: dictionary coding is inconsistent. Fix: enforce `--MODIFY` and `--DECOD` usage per domain, use PESTRESC for PE coding, and record dictionary metadata in Define-XML.
- [ ] Problem: `--PRESP/--OCCUR/--STAT/--REASND` rules are not enforced. Fix: validate prespecified vs occurrence logic, require `--STAT="NOT DONE"` to have `--REASND`, and disallow contradictory values.
- [ ] Problem: baseline and `--LOBXFL` logic is missing. Fix: calculate baseline/LOBXFL using RFXSTDTC and ensure a single last pre-treatment record per test per subject.
- [ ] Problem: domain pair and FA relationships are not enforced. Fix: validate parent-child linking for MB/MS and PC/PP, enforce RELREC dataset-level relationships, and validate FA dataset naming when split by parent domain.

## Trial design model alignment (Ch 7)
- [ ] Problem: TAETORD/ETCD/EPOCH constraints are not validated. Fix: require TAETORD integer ordering within ARMCD, enforce ETCD length <= 8, and ensure EPOCH values are consistent across arms.
- [ ] Problem: TA/TE/SE/DM alignment is not enforced. Fix: require TA ETCD to exist in TE, ensure DM ARM/ARMCD matches TA, and align SE ETCD/EPOCH and ACTARMCD derivation with TA/TE.
- [ ] Problem: TV is not used to validate VISIT variables. Fix: require TV when VISIT/VISITNUM appear in subject domains and validate SV planned visits against TV.
- [ ] Problem: TI/IE linkage is not enforced. Fix: ensure IE records reference TI IETESTCD/IETEST values and require TI definitions for all IE criteria.
- [ ] Problem: TS/TD/TM datasets are not generated/validated. Fix: implement Trial Summary (TS), Trial Disease Assessments (TD), and Trial Disease Milestones (TM) with required parameters and unique keys.

## Study reference datasets (Ch 9)
- [ ] Problem: Device Identifiers (DI) are not implemented. Fix: implement DI per SDTMIG-MD and require SPDEVID values in subject domains to exist in DI.
- [ ] Problem: Non-host Organism Identifiers (OI) are not implemented. Fix: implement OI per spec, enforce NHOID uniqueness, OIPARMCD/OIPARM/OIVAL completeness, hierarchical OISEQ order, and forbid extra variables.

## Appendix-driven naming and CT checks (Ch 10)
- [ ] Problem: standard SUPP QNAM codes are not enforced. Fix: validate QNAMs against Appendix C1 and mark nonstandard QNAMs with `def:IsNonStandard`.
- [ ] Problem: naming fragments are not used for `--TESTCD`/QNAM. Fix: implement a fragment-based generator/hint system (Appendix D) to keep codes <= 8 chars and avoid collisions.
- [ ] Problem: CT release provenance is not recorded. Fix: store CT release version/publishing set in the standards registry and emit it in Define-XML with strict CT validation.

## Ingest and mapping
- [ ] Problem: `CsvTable` loads entire datasets into memory. Fix: use Polars CSV streaming for data and only sample rows for hints in `crates/sdtm-ingest/src/csv_table.rs`.
- [ ] Problem: header detection is heuristic-only. Fix: allow explicit header row index or schema hints to override heuristics.
- [ ] Problem: mapping suggestions are not persisted. Fix: store mapping configs and reuse them across runs (like Python legacy) via a mapping repository.
- [ ] Problem: mapping engine uses only column name tokens. Fix: incorporate variable labels, synonyms, and domain patterns from standards in `crates/sdtm-standards/src/loaders.rs`.

## Domain processing and shared utilities
- [ ] Problem: duplicate `any_to_string` and numeric parsing logic. Fix: consolidate into a single utility module and remove duplicates from `crates/sdtm-core/src/domain_processors/common.rs`, `crates/sdtm-core/src/relationships.rs`, and `crates/sdtm-report/src/lib.rs`.
- [ ] Problem: repeated case conversions and column lookups. Fix: add a case-insensitive column lookup helper used across `processor`, `preprocess`, and `validate`.
- [ ] Problem: CT resolution uses multiple inconsistent algorithms. Fix: standardize CT normalization via a single API with strict and explicit modes.
- [ ] Problem: preprocess rules are monolithic and hard to audit. Fix: split `crates/sdtm-core/src/preprocess.rs` into per-domain modules and drive with a rule table.
- [ ] Problem: heuristic fills can add data without provenance. Fix: remove or require explicit mapping metadata before populating any value.
- [ ] Problem: SDY derivation fails without reference starts. Fix: build a `reference_starts` map from DM (RFSTDTC) and pass it into `ProcessingContext`.
- [ ] Problem: date normalization is duplicated. Fix: create a shared date module and run a single normalization pass.
- [ ] Problem: per-row loops are slow and allocation-heavy. Fix: use Polars expressions for trimming/uppercasing where possible.

## Validation refactor
- [ ] Problem: cross-domain validations are missing. Fix: add checks for split SEQ uniqueness, SUPP QNAM uniqueness, QVAL non-empty, and relationship key integrity.
- [ ] Problem: missing dataset detection depends on parsing rule text. Fix: add explicit rule metadata mapping in `crates/sdtm-standards/src/loaders.rs` and use it in `crates/sdtm-validate/src/lib.rs`.
- [ ] Problem: issue reporting is unstructured. Fix: add a structured issue model with counts and samples and update `crates/sdtm-cli/src/summary.rs` to render it.

## Output generation
- [ ] Problem: Define-XML reloads CT registry internally. Fix: pass `CtRegistry` into `write_define_xml` and `write_dataset_xml`.
- [ ] Problem: Dataset-XML is built in memory. Fix: stream dataset XML writing directly from frames.
- [ ] Problem: split dataset names are ignored in outputs. Fix: use `DomainFrameMeta.dataset_name` for XPT/XML file names and metadata.

## Define-XML (MSG v2.0 sample parity)
- [ ] Problem: no `<?xml-stylesheet?>` PI in define.xml. Fix: emit a stylesheet PI referencing `define2-1.xsl`.
- [ ] Problem: `<def:Standards>` section is missing. Fix: emit IG/CT standards with Name, Type, Version, Status, PublishingSet from SDTMIG/CT versions.
- [ ] Problem: aCRF/cSDRG references are absent. Fix: add `<def:AnnotatedCRF>` and `<def:SupplementalDoc>` with `<def:DocumentRef>` entries when supplied.
- [ ] Problem: dataset/document leafs are missing. Fix: emit `<def:leaf>` for each dataset/document and link via `def:ArchiveLocationID` on `ItemGroupDef`.
- [ ] Problem: ItemGroupDef lacks Purpose, StandardOID, and Class element; Repeating is always Yes. Fix: populate `Purpose`, `def:StandardOID`, add `<def:Class Name=...>`, and set `Repeating` based on domain structure.
- [ ] Problem: ItemRef lacks Role and Method references. Fix: populate `Role` from `Variables.csv` and `MethodOID` when a derivation is present.
- [ ] Problem: ItemDef is missing SASFieldName and uses generic float types. Fix: add `SASFieldName` and choose `integer` vs `float` based on data characteristics.
- [ ] Problem: Origin metadata is missing. Fix: emit `<def:Origin>` with Type/Source and `<def:DocumentRef>` for CRF-based items.
- [ ] Problem: Value-level metadata is not represented. Fix: implement `<def:ValueListDef>` and `<def:WhereClauseDef>` for value-level items (e.g., ORRES/ORRESU, SUPP QNAM/QVAL) using a value-level rules table.
- [ ] Problem: CodeList items lack Alias/NonStandard flags. Fix: add `Alias` (NCI codes) and `def:IsNonStandard` where appropriate.
- [ ] Problem: External dictionaries are not represented. Fix: emit `<ExternalCodeList>` for MedDRA/SNOMED/ISO when used.
- [ ] Problem: leaf links ignore output format. Fix: point leafs to Dataset-XML or XPT depending on configured outputs.

## Dataset-XML (MSG v2.0 sample parity)
- [ ] Problem: Dataset-XML root attributes differ from MSG. Fix: set `data:DatasetXMLVersion="1.0.0"`, `SourceSystem`, and `SourceSystemVersion` to configured values.
- [ ] Problem: FileOID format differs from MSG. Fix: use `{DefineFileOID}(IG.{base_domain}).Data({dataset_name})` and set `PriorFileOID` to define.xml.
- [ ] Problem: split datasets can emit wrong ItemGroupOID. Fix: for split datasets (e.g., LBCH) emit `ItemGroupOID="IG.LB"` while using split dataset name in `Data(...)` and file name.
- [ ] Problem: ReferenceData vs ClinicalData containers are not enforced. Fix: choose container based on domain class from standards.
- [ ] Problem: MSG-style header comments are missing. Fix: add optional comments (dataset name + source) without affecting XML validity.
- [ ] Problem: item order can drift from SDTM variable order. Fix: order ItemData by Variable Order from `Variables.csv`.
- [ ] Problem: numeric output can use scientific notation. Fix: format numeric strings as plain decimals.
- [ ] Problem: split dataset outputs are not stored in `split/`. Fix: mirror MSG directory layout for split datasets.

## XPT (SAS V5 transport) MSG v2.0 sample parity
- [ ] Problem: XPT headers are emitted with fixed defaults (`SAS 9.4`, `OS RUST`, static timestamps), which do not match MSG samples. Fix: plumb header metadata (SAS version, OS name, created/modified) from config or study metadata into `XptWriterOptions` in `crates/sdtm-xpt/src/lib.rs` and `crates/sdtm-report/src/lib.rs`, and assert deterministic values for strict outputs.
- [ ] Problem: dataset name defaults to the base domain when split datasets exist, so `LBCH` becomes `LB`. Fix: carry a dataset_name through `DomainFrameMeta` and use it in `build_xpt_dataset` and file naming; base domain stays in metadata only.
- [ ] Problem: dataset label is auto-filled with dataset name when missing; MSG split datasets leave labels blank. Fix: write the label only when explicitly defined in standards or input metadata; otherwise leave the XPT label blank (spaces).
- [ ] Problem: non-ASCII characters are silently replaced with `?` in `pad_ascii`, which hides invalid input. Fix: in strict mode, reject any non-ASCII value and surface a validation error instead of mutating data.
- [ ] Problem: column lengths are derived from observed data, which can drift across runs and diverge from Define-XML. Fix: compute lengths once (from standards overrides or explicit config) and reuse the same lengths for XPT, Define-XML, and Dataset-XML.
- [ ] Problem: SDTMIG length limits are not enforced (values can be truncated by padding). Fix: validate limits (e.g., 8-char identifiers, 40-char labels, 200-char text) and fail fast with explicit errors before writing XPT.
- [ ] Problem: numeric length can be overridden or shortened, risking precision loss. Fix: force all numeric columns to length 8 and ignore smaller overrides.
- [ ] Problem: numeric values rendered into character columns may use scientific notation via `to_string()`. Fix: add a fixed decimal renderer for numeric-to-char conversions to match MSG style.
- [ ] Problem: there is no parity check between XPT and Define-XML. Fix: add a validation step that compares dataset name, column order, labels, and lengths in XPT against Define-XML, using MSG samples in `tests/validation/data/xpt`.
- [ ] Problem: split dataset outputs are written to a flat `xpt/` directory. Fix: mirror MSG layout by writing split datasets to `xpt/split/` (e.g., `xpt/split/lbch.xpt`).

## Performance and memory
- [ ] Problem: ordered maps are used in hot paths without needing order. Fix: replace `BTreeMap`/`BTreeSet` with `HashMap`/`HashSet` in hot code paths (e.g., `crates/sdtm-core/src/processor.rs`, `crates/sdtm-map/src/engine.rs`).
- [ ] Problem: repeated `to_uppercase` allocations. Fix: cache normalized keys and reuse across steps.
- [ ] Problem: DataFrames are cloned unnecessarily. Fix: use mutable references and in-place transforms in `crates/sdtm-cli/src/commands.rs`.

## Testing and quality
- [ ] Problem: no tests for split dataset naming parity. Fix: add tests for QS36/FACM dataset naming across outputs and SUPP datasets.
- [ ] Problem: no tests for long-text SUPP splitting. Fix: add tests for >200 char splitting per SDTMIG Section 4.5.3.
- [ ] Problem: no tests for RELREC record-level vs dataset-level behavior. Fix: add parity tests for RELTYPE usage.
- [ ] Problem: no tests for SDY computation via DM reference starts. Fix: add tests for SDY with `reference_starts`.

## Python removal (full cutover)
- [ ] Problem: parity with legacy Python is not tracked. Fix: build a parity checklist against `cdisc_transpiler/` and mark Rust coverage before removal.
- [ ] Problem: full Python codebase still ships in the repo. Fix: remove `cdisc_transpiler/`, delete Python entry points, and remove packaging files (`pyproject.toml`, `requirements.txt`, `cdisc_transpiler.egg-info`) once parity is confirmed.
- [ ] Problem: Rust still references Python or fallback paths. Fix: delete any Python interop hooks, config flags, or tests that depend on Python output.
- [ ] Problem: no clear cutover plan. Fix: tag a final Python release, archive it to a separate `legacy/` repo or release asset, and document the migration path.
- [ ] Problem: CI/docs still mention Python. Fix: remove Python CI jobs, update `README.md` and docs to Rust-only, and communicate Python deprecation.
