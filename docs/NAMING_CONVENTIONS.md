# Naming Conventions (CDISC Transpiler)

This repository is converging on a **Ports & Adapters (Hexagonal / Clean
Architecture)** shape. Naming is part of enforcing boundaries: names should tell
you **which layer** you’re in and **what role** a type plays.

## Goals

- Make dependency direction obvious from names.
- Keep the public CLI stable while refactoring internals.
- Avoid vague “grab bag” naming unless narrowly scoped and layer-specific.

## General Python Rules

- **Packages / modules**: `snake_case`.
- **Classes / Protocols / Enums**: `PascalCase`.
- **Functions / methods / variables**: `snake_case`.
- **Constants**: `UPPER_SNAKE_CASE`.
- Prefer explicit names over abbreviations (except well-known standards below).

## Standard Abbreviations (Allowed)

Use these consistently (do not invent new variants):

- `SDTM`, `ADaM`, `ODM`, `XPT`, `XML`, `CT`

In Python identifiers:

- Prefer `DomainDefinitionRepositoryPort` over `DomainDefinitionPort` when it is
  a repository-style port.
- Prefer `domain_code` over ambiguous names like `code`.

## Layer-Specific Naming

### Domain (`cdisc_transpiler/domain/`)

- Pure business logic; no I/O.
- **Entities / value objects**: noun phrases, e.g. `StudyMetadata`,
  `ControlledTerminology`.
- **Domain services**: `*Service` only when they represent real domain
  capabilities (not generic orchestration).
- Avoid vague catch-alls like `utils` unless narrowly scoped (e.g.
  `xml_value_formatting.py`).

### Application (`cdisc_transpiler/application/`)

- Orchestrates workflows via ports.

**Use cases**

- Name as `*UseCase` (e.g. `StudyProcessingUseCase`).
- Public entry point method: `execute(request: *Request) -> *Response`.
- Prefer explicit injected dependency names:
  - `domain_definition_repository` rather than `domain_definitions`
  - `study_data_repository` rather than `repo`

**Ports**

- All ports must end with `Port`.
- Repository-style ports should end with `RepositoryPort`.
  - Example: `CTRepositoryPort`, `DomainDefinitionRepositoryPort`,
    `StudyDataRepositoryPort`.
- Service-style ports should end with `Port` or `ServicePort` if it clarifies
  intent.
- Writer ports should end with `WriterPort`.

### Infrastructure (`cdisc_transpiler/infrastructure/`)

- Implements ports with concrete I/O and adapters.

**Adapters / implementations**

Use one of:

- `*Adapter` when the class is primarily translating between layers.
- `*Repository` for repository implementations.
- `*Writer` for file writers.

Examples:

- `DomainDiscoveryServiceAdapter`
- `StudyDataRepository`
- `XPTWriter`

### CLI (`cdisc_transpiler/cli/`)

- Driver adapter; parses args and calls use cases.
- Keep click command names/flags stable.
- Prefer names like `presenters/*` and `commands/*` over generic `utils`.

## Parameter Naming Guidelines

- Prefer:
  - `domain_code` for SDTM domain identifier (`"DM"`, `"AE"`)
  - `dataset_name` for dataset name in outputs (`"QSSL"`, `"LBHE"`)
  - `study_id` for `StudyOID`-like identifiers
  - `output_dir` / `output_path` (directory vs file)
- Avoid:
  - `code` (too generic)
  - `name` (unless scoped, e.g. `variable_name`)

## Avoid Vague Names

Avoid introducing new:

- `utils`, `helpers`, `manager`, `processor`

unless narrowly scoped and clearly layer-specific.

If a module truly is a small, cohesive collection of helpers, prefer a name
describing what it helps with, e.g.

- `xml_utils.py` (XML-related helpers only)
- `dataset_xml/value_formatting.py`

## Examples

### Good

- `DomainDefinitionRepositoryPort.get_domain(domain_code: str)`
- `StudyProcessingUseCase.execute(request: ProcessStudyRequest)`
- `DomainDiscoveryServiceAdapter` (implements `DomainDiscoveryPort`)

### Avoid

- `DomainDefinitionPort.get_domain(code: str)` (ambiguous)
- `DataManager`, `Processor`, `Helpers` (unclear role/scope)

## Refactor Rule of Thumb

When renaming, prefer:

1. Rename types/parameters first (low churn, high clarity)
2. Update call sites
3. Only then consider file/package renames (high churn)
