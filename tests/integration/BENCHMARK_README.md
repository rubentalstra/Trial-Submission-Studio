# Performance Benchmarks (Rust)

This repository is moving benchmarking to the Rust implementation. The Python
benchmarks are legacy and should not be expanded.

## Running Benchmarks

### Run all benchmarks

```bash
cargo bench
```

### Run benchmarks for a specific crate

```bash
cargo bench -p sdtm-core
```

### Run a specific benchmark

```bash
cargo bench --bench study_processing
```

## Suggested Benchmark Coverage

- Full study processing (small and large mock studies)
- Mapping engine (column matching + hints)
- Dataset-XML writer (streaming vs non-streaming)
- XPT writer (column ordering + type conversion)

## Benchmark Framework

Use `criterion` for statistical benchmarking and optional `iai-callgrind` for
instruction-level profiling. Keep benchmarks deterministic and use committed
mock data sets from `mockdata/`.

## CI/CD Integration

```yaml
- name: Run Benchmarks
  run: cargo bench
```

## Best Practices

- Run benchmarks on stable hardware and quiet machines
- Keep inputs fixed and versioned
- Track baselines for regression detection
