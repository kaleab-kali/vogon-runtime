# Contributing

Thanks for helping improve Vogon Runtime.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo check -p vogon-cli --no-default-features
cargo +1.85.0 test --workspace --all-features --locked
cargo bench -p vogon-core --bench runtime -- --iterations 100
cargo build --release --workspace --all-features
cargo run --release -p vogon-cli -- check --json fixtures/workflows/support-triage.toml
cargo run --release -p vogon-cli -- verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo run --release -p vogon-cli -- verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force
target/install-smoke/bin/vogon --version
target/install-smoke/bin/vogon check --json fixtures/workflows/support-triage.toml
target/install-smoke/bin/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
target/install-smoke/bin/vogon verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo package --workspace --allow-dirty --no-verify --offline
```

The package command validates the crate archives that would be prepared for
publication. Use `--offline` when working without registry access after
dependencies have already been fetched.

### Windows target file locks

On Windows, local antivirus or indexers can briefly hold files in `target` while
Cargo is rebuilding. If a check fails with `os error 32` while removing an
object file, rerun the same command with a single build job:

```powershell
$env:CARGO_BUILD_JOBS='1'
cargo test --workspace --all-features
```

Use the same environment variable for `cargo build` or other Cargo commands
that hit the same file-lock error.

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
- `RustSec advisory audit` when Rust dependency files or the audit workflow
  change

The optional `Live Gemini Smoke` workflow can be run manually by maintainers
when `GEMINI_API_KEY` is configured as a repository or environment secret. It is
not required for ordinary pull requests because deterministic CI must not
depend on external provider availability.

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
