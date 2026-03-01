# Performance Budget (v1.0)

## CI smoke thresholds (conservative)

- Geom intersection hot path (`intersect_perf_threshold_smoke`):
  - 20k line-circle intersection loop should complete < 2500ms on CI baseline.
- Nesting medium deterministic case:
  - completes under configured `RunLimits` (`time_limit_ms <= 500`, `iteration_limit <= 50`).

## Benchmark harnesses

- `core/crates/diycad_geom/benches/geom_bench.rs`
- `core/crates/diycad_nesting/benches/nesting_bench.rs`

Benchmarks are for trend monitoring (manual/CI artifact), while smoke thresholds are hard CI gates.
