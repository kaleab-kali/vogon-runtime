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

## Runtime Cache

`vogon run` can persist a bounded cache of provider outputs across repeated
runs:

```sh
cargo run -p vogon-cli -- run --cache-file target/vogon.cache.json fixtures/workflows/support-triage.toml
```

The cache is scoped by adapter identity and stable step input hash, so entries
are not reused across different providers, endpoints, models, or input hashes.
Use `--cache-max-entries` to choose the retained entry count; the default is
`1024`, and `0` disables storage.

Treat cache files as sensitive. They may contain raw model outputs, including
values redacted from replay files, and should stay out of version control.
