# Performance

Vogon Runtime includes a stable benchmark smoke target for the provider-neutral
runtime path. The benchmark does not enforce timing thresholds because CI hosts
vary, but it provides a repeatable command for comparing changes.

Run the benchmark locally:

```sh
cargo bench -p vogon-core --bench runtime -- --iterations 1000
```

The benchmark prints:

- `iterations`: number of workflow runs.
- `elapsed_ms`: total elapsed wall-clock time.
- `iterations_per_second`: completed workflow runs per second.

Use the same machine, Rust toolchain, and iteration count when comparing two
branches.

CI validates the benchmark smoke output shape with:

```sh
cargo bench -p vogon-core --bench runtime -- --iterations 100 | python scripts/check_benchmark_output.py --expected-iterations 100
```

The validator checks the expected iteration count and requires positive finite
`elapsed_ms` and `iterations_per_second` values. It intentionally does not set a
minimum throughput threshold because GitHub-hosted runner performance varies.
