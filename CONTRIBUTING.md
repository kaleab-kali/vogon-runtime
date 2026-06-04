# Contributing

Thanks for helping improve Vogon Runtime.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo +1.85.0 test --workspace --all-features --locked
cargo bench -p vogon-core --bench runtime -- --iterations 100
cargo build --release --workspace --all-features
cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo run --release -p vogon-cli -- verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo package --workspace --allow-dirty --no-verify
```

The package command validates the crate archives that would be prepared for
publication. Use `--offline` when working without registry access after
dependencies have already been fetched.

## Pull Requests

- Keep changes focused and reviewable.
- Add or update tests for behavior changes.
- Update docs when user-facing behavior changes.
- Keep provider-specific code outside `vogon-core`.
- Do not commit secrets, credentials, or real customer data.

The `main` branch is protected. Pull requests must pass:

- `Rust workspace`
- `Windows release smoke`
- `Dependency review`
- `Minimum supported Rust`

Maintainers merge accepted pull requests with regular merge commits. Squash
merges are not used for this repository, and merged topic branches may remain
available for auditability.

## Commit Style

Use short imperative commit messages, for example:

```text
Add workspace scaffold
Implement deterministic echo adapter
Document replay format
```
