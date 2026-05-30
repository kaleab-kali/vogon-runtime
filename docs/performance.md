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
