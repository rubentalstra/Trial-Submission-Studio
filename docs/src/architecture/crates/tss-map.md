# tss-map

Fuzzy column mapping engine crate.

## Overview

`tss-map` provides intelligent matching between source columns and SDTM variables.

## Responsibilities

- Fuzzy string matching for column names
- Match confidence scoring
- Mapping suggestions
- Type compatibility checking

## Dependencies

```toml
[dependencies]
rapidfuzz = "0.5"
tss-standards = { path = "../tss-standards" }
tss-model = { path = "../tss-model" }
```

## Architecture

### Module Structure

```
tss-map/
├── src/
│   ├── lib.rs
│   ├── matcher.rs      # Fuzzy matching logic
│   ├── scorer.rs       # Confidence scoring
│   ├── mapping.rs      # Mapping structures
│   └── suggestions.rs  # Auto-suggestion engine
```

## Matching Algorithm

### Process

1. **Normalize names** - Case folding, remove special chars
2. **Calculate similarity** - Multiple algorithms
3. **Apply domain hints** - Boost relevant matches
4. **Score confidence** - Combine factors
5. **Rank suggestions** - Order by score

### Similarity Metrics

```rust
pub fn calculate_similarity(source: &str, target: &str) -> f64 {
    let ratio = rapidfuzz::fuzz::ratio(source, target);
    let partial = rapidfuzz::fuzz::partial_ratio(source, target);
    let token_sort = rapidfuzz::fuzz::token_sort_ratio(source, target);

    // Weighted combination
    (ratio * 0.4 + partial * 0.3 + token_sort * 0.3) / 100.0
}
```

### Confidence Levels

| Score     | Level  | Action      |
|-----------|--------|-------------|
| > 0.80    | High   | Auto-accept |
| 0.50-0.80 | Medium | Review      |
| < 0.50    | Low    | Manual      |

## API

### Finding Matches

```rust
use tss_map::{Matcher, MatchOptions};

let matcher = Matcher::new( & standards);
let suggestions = matcher.suggest_mappings(
& source_columns,
domain,
MatchOptions::default ()
) ?;

for suggestion in suggestions {
println!("{} -> {} ({:.0}%)",
         suggestion.source,
         suggestion.target,
         suggestion.confidence * 100.0
);
}
```

### Mapping Structure

```rust
pub struct Mapping {
    pub source_column: String,
    pub target_variable: String,
    pub confidence: f64,
    pub user_confirmed: bool,
}
```

### Match Options

```rust
pub struct MatchOptions {
    pub min_confidence: f64,
    pub max_suggestions: usize,
    pub consider_types: bool,
}
```

## Heuristics

### Domain-Specific Boosting

| Pattern  | Domain | Boost |
|----------|--------|-------|
| `*SUBJ*` | All    | +0.1  |
| `*AGE*`  | DM     | +0.15 |
| `*TERM*` | AE, MH | +0.15 |
| `*TEST*` | LB, VS | +0.15 |

### Common Transformations

| Source Pattern | Target  |
|----------------|---------|
| SUBJECT_ID     | USUBJID |
| PATIENT_AGE    | AGE     |
| GENDER         | SEX     |
| VISIT_DATE     | --DTC   |

## Testing

```bash
cargo test --package tss-map
```

### Test Categories

- Exact match detection
- Fuzzy match accuracy
- Confidence scoring
- Domain-specific matching

## See Also

- [Column Mapping](../../user-guide/column-mapping.md) - User guide
- [tss-standards](tss-standards.md) - Variable definitions
