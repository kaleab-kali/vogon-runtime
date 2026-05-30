# Release Process

Vogon Runtime releases are cut from `main` after CI is green and the changelog
has been updated for the version being shipped.

## Before Tagging

Run the full local verification set:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --release --workspace --all-features
cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo install --path crates/vogon-cli --locked --root target/install-smoke --force
cargo doc --workspace --all-features --no-deps
cargo package --workspace --allow-dirty --no-verify
```

Confirm that:

- `CHANGELOG.md` describes the release.
- `Cargo.lock` is committed.
- The GitHub Actions CI workflow has passed on `main`.
- No private prompts, credentials, secrets, or sensitive replay data are present.

After the install smoke command, run the installed binary for your platform:

```sh
target/install-smoke/bin/vogon --version
```

On Windows, use `target\install-smoke\bin\vogon.exe --version`.

## Tagging

Create a semantic version tag from `main`:

```sh
git checkout main
git pull --ff-only
git tag v0.1.0
git push origin v0.1.0
```

Use the real version number in place of `v0.1.0`.

## Automated Release Artifact

Pushing a `v*.*.*` tag starts the Release workflow. The workflow:

- Builds `vogon-cli` in release mode with the committed lockfile.
- Runs the optimized CLI against a replay fixture.
- Packages a Linux x86_64 `vogon` binary as a `.tar.gz` archive.
- Creates a GitHub release for the tag and uploads the archive.

## Manual Publishing

Crate publishing to crates.io is intentionally manual while the public API is
pre-release. Before publishing, verify package contents locally and publish the
workspace crates in dependency order.
