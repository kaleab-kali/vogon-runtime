## Summary

-

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo check -p vogon-cli --no-default-features`
- [ ] `python -m unittest scripts.test_write_spdx_sbom`
- [ ] `python -m unittest scripts.test_check_docs_links`
- [ ] `python scripts/check_docs_links.py --root .`
- [ ] `cargo +1.85.0 test --workspace --all-features --locked`
- [ ] `cargo bench -p vogon-core --bench runtime -- --iterations 100`
- [ ] `cargo build --release --workspace --all-features`
- [ ] `cargo run --release -p vogon-cli -- check --json fixtures/workflows/support-triage.toml`
- [ ] `cargo run --release -p vogon-cli -- verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `cargo run --release -p vogon-cli -- verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json`
- [ ] `cargo run --release -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json`
- [ ] `cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force`
- [ ] `target/install-smoke/bin/vogon --version`
- [ ] `target/install-smoke/bin/vogon check --json fixtures/workflows/support-triage.toml`
- [ ] `target/install-smoke/bin/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `target/install-smoke/bin/vogon verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json`
- [ ] `docker build --tag vogon-runtime:smoke .`
- [ ] `docker run --rm vogon-runtime:smoke --version`
- [ ] `docker run --rm -v "$PWD:/work" vogon-runtime:smoke check --json fixtures/workflows/support-triage.toml`
- [ ] `docker run --rm -v "$PWD:/work" vogon-runtime:smoke verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json`
- [ ] `docker run --rm -v "$PWD:/work" vogon-runtime:smoke trace --jsonl fixtures/replays/support-triage.replay.json`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`
- [ ] `cargo package --workspace --allow-dirty --no-verify --offline`
- [ ] Relevant CLI smoke test:
- [ ] RustSec advisory audit passed or is not affected by this change.

## Checklist

- [ ] The change is focused and reviewable.
- [ ] Tests were added or updated for behavior changes.
- [ ] Public docs were updated for user-facing behavior changes.
- [ ] Provider-specific code remains outside `vogon-core`.
- [ ] No secrets, credentials, private prompts, or sensitive replay data are included.

## Notes

-
