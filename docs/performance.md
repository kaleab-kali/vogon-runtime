# Performance

Vogon Runtime includes a stable benchmark smoke target for the provider-neutral
runtime path. The benchmark does not enforce timing thresholds because CI hosts
vary, but it provides a repeatable command for comparing changes.

Run the benchmark locally:

```sh
cargo bench -p vogon-core --bench runtime --locked -- --iterations 1000
```

The benchmark prints:

- `iterations`: number of workflow runs.
- `elapsed_ms`: total elapsed wall-clock time.
- `iterations_per_second`: completed workflow runs per second.

Use the same machine, Rust toolchain, and iteration count when comparing two
branches.

Release builds use thin link-time optimization, one code generation unit, and
symbol stripping to keep shipped CLI artifacts optimized and smaller without
changing runtime behavior.

CI validates the benchmark smoke output shape with:

```sh
cargo bench -p vogon-core --bench runtime --locked -- --iterations 100 | cargo run -p vogon-xtask -- check-benchmark-output --expected-iterations 100 --max-elapsed-ms 10000
```

The validator checks the expected iteration count, requires positive finite
`elapsed_ms` and `iterations_per_second` values, and rejects runs over a loose
10,000 ms safety budget. This ceiling is intended to catch hangs or catastrophic
regressions, not small throughput changes; GitHub-hosted runner performance is
too variable for a tight microbenchmark threshold.

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

## Runtime Output Bounds

Provider adapters cap successful HTTP response bodies at 1 MiB before JSON
decoding. This limits memory use when a provider or compatible endpoint returns
an unexpectedly large response.

Persisted replay and run-cache files use the same 1 MiB ceiling as Vogon's
workflow, replay, and cache readers. Oversized writes fail before replacing an
existing artifact, so Vogon does not create files that it will later refuse to
open. Runs written directly to standard output are not persisted artifacts and
are unaffected by the file limit.
