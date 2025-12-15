# Performance Benchmarks

This directory contains performance benchmark tests using `pytest-benchmark`.

## Running Benchmarks

### Run all benchmarks:
```bash
pytest tests/integration/test_performance_benchmarks.py --benchmark-only
```

### Run benchmarks with comparison:
```bash
# First run - save baseline
pytest tests/integration/test_performance_benchmarks.py --benchmark-only --benchmark-save=baseline

# Later runs - compare against baseline
pytest tests/integration/test_performance_benchmarks.py --benchmark-only --benchmark-compare=baseline
```

### Run specific benchmark:
```bash
pytest tests/integration/test_performance_benchmarks.py::TestStudyProcessingPerformance::test_benchmark_small_study_processing --benchmark-only
```

### Generate detailed statistics:
```bash
pytest tests/integration/test_performance_benchmarks.py --benchmark-only --benchmark-verbose
```

### Save results to JSON:
```bash
pytest tests/integration/test_performance_benchmarks.py --benchmark-only --benchmark-json=benchmark_results.json
```

## Benchmark Markers

Benchmarks are marked with `@pytest.mark.benchmark` so they can be easily selected or excluded:

```bash
# Run only benchmarks
pytest -m benchmark --benchmark-only

# Skip benchmarks in regular test runs
pytest -m "not benchmark"
```

## What's Benchmarked

### Study Processing Performance
- **Small study** (DEMO_CF): ~11 domains, 59 records
  - Expected baseline: <5 seconds
  - Tests complete pipeline: discovery → processing → XPT generation

- **Large study** (DEMO_GDISC): ~18 domains, 260 records
  - Expected baseline: <20 seconds
  - Tests complete pipeline with variant domains

### Domain Processing Performance
- **DM domain**: Demographics (largest/most complex)
  - Expected baseline: <2 seconds

- **AE domain**: Adverse Events (moderate complexity)
  - Expected baseline: <1 second

### Transformation Performance
- **Wide-to-long transformation**: Findings data
  - Expected baseline: <100ms for 1000 rows
  - Common and potentially expensive operation

## Understanding Results

pytest-benchmark provides detailed statistics:

```
Name (time in s)                                    Min       Max      Mean    StdDev   Median
test_benchmark_small_study_processing           2.5000    3.0000    2.7500    0.2041   2.7000
test_benchmark_large_study_processing          15.0000   18.0000   16.5000    1.2247  16.2000
```

Key metrics:
- **Min/Max**: Fastest and slowest execution times
- **Mean**: Average execution time
- **StdDev**: Standard deviation (consistency)
- **Median**: Middle value (often more reliable than mean)

## Performance Regression Detection

To detect performance regressions:

1. **Establish baseline** on main branch:
   ```bash
   pytest tests/integration/test_performance_benchmarks.py --benchmark-only --benchmark-save=main
   ```

2. **Compare feature branch** against baseline:
   ```bash
   pytest tests/integration/test_performance_benchmarks.py --benchmark-only --benchmark-compare=main
   ```

3. **Review differences**: pytest-benchmark will show % changes and highlight regressions.

## CI/CD Integration

Add to CI pipeline:

```yaml
- name: Run Performance Benchmarks
  run: |
    pytest tests/integration/test_performance_benchmarks.py \
      --benchmark-only \
      --benchmark-json=benchmark_results.json
    
- name: Compare Against Baseline
  run: |
    pytest tests/integration/test_performance_benchmarks.py \
      --benchmark-only \
      --benchmark-compare=baseline \
      --benchmark-compare-fail=mean:10%  # Fail if >10% slower
```

## Benchmark Configuration

Benchmarks can be configured via command line:

```bash
# Run each benchmark at least 5 times
pytest --benchmark-min-rounds=5

# Disable GC during benchmarks for consistency
pytest --benchmark-disable-gc

# Warm up before timing
pytest --benchmark-warmup=on

# Set timeout per benchmark
pytest --benchmark-max-time=60
```

Or in `pyproject.toml`:

```toml
[tool.pytest.ini_options]
benchmark_min_rounds = 5
benchmark_disable_gc = true
benchmark_warmup = true
```

## Best Practices

1. **Run benchmarks in isolation**: Use `--benchmark-only` to skip regular tests
2. **Stable environment**: Run on consistent hardware, minimize background processes
3. **Multiple rounds**: Let pytest-benchmark run multiple rounds for statistical accuracy
4. **Save baselines**: Keep baseline results for comparison
5. **Track over time**: Monitor trends to catch gradual performance degradation
6. **Set thresholds**: Use `--benchmark-compare-fail` to enforce performance standards

## Interpreting Performance

**Good performance indicators:**
- Small study (<5s): Fast feedback for developers
- Large study (<20s): Reasonable for CI/CD
- Low StdDev: Consistent, predictable performance
- Linear scaling: Performance scales with data size

**Red flags:**
- Sudden increases (>20%): Investigate regression
- High StdDev: Inconsistent, unpredictable performance
- Non-linear scaling: May indicate algorithmic issues

## Adding New Benchmarks

When adding new benchmarks:

1. Mark with `@pytest.mark.benchmark`
2. Use `benchmark` fixture to wrap the code to measure
3. Include verification assertions (result should be valid)
4. Document expected baseline in docstring
5. Use realistic data sizes
6. Isolate the operation being measured

Example:
```python
@pytest.mark.benchmark
def test_benchmark_new_operation(benchmark):
    """Benchmark new operation.
    
    Expected baseline: <1 second for 1000 items.
    """
    def operation():
        # Code to benchmark
        return process_data(large_dataset)
    
    result = benchmark(operation)
    assert result is not None, "Should produce result"
```
