# Release Process

Vogon Runtime releases are cut from `main` after CI is green and the changelog
has been updated for the version being shipped.

## Before Tagging

Run the full local verification set:

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

Confirm that:

- `CHANGELOG.md` describes the release.
- `Cargo.lock` is committed.
- The GitHub Actions CI workflow has passed on `main`.
- The Security Audit workflow has a recent successful run for the committed
  `Cargo.lock`.
- If provider adapter or deployment behavior changed, the optional `Live Gemini
  Smoke` workflow has passed with `GEMINI_API_KEY` configured as a repository or
  environment secret.
- No private prompts, credentials, secrets, or sensitive replay data are present.

GitHub Actions workflows set `CARGO_NET_RETRY=10` for Cargo commands so
transient registry resets are retried before a job fails.

After the install smoke command, run the installed binary for your platform
against the same workflow and replay fixtures. The install smoke uses
`--offline` because earlier verification commands have already fetched and
built the locked dependencies:

```sh
target/install-smoke/bin/vogon --version
target/install-smoke/bin/vogon check --json fixtures/workflows/support-triage.toml
target/install-smoke/bin/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
target/install-smoke/bin/vogon verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
```

On Windows, use `target\install-smoke\bin\vogon.exe` for the installed binary
commands.

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

- Builds `vogon-cli` in release mode with the committed lockfile on Linux and
  Windows.
- Runs machine-readable `check` and `verify` smoke tests against each optimized
  CLI artifact.
- Runs each optimized CLI artifact against every committed replay fixture.
- Packages a Linux x86_64 `vogon` binary as a `.tar.gz` archive.
- Packages a Windows x86_64 `vogon.exe` binary as a `.zip` archive.
- Includes `README.md` and `LICENSE` in each CLI archive.
- Writes SHA-256 checksum files for each archive.
- Generates GitHub artifact attestations for each release archive.
- Creates a GitHub release for the tag and uploads both archives with their
  checksum files.

The Release workflow can also be run manually from GitHub Actions to dry-run
the Linux and Windows artifact builds from a branch. Manual runs upload the
archives and checksum files as workflow artifacts, but they do not create a
GitHub release. Manual runs also download the uploaded archives and verify their
checksum files so artifact download behavior is covered before tag publishing.

## Verifying Release Downloads

Each release archive is published with a `.sha256` file. Download the archive
and its matching checksum file before extracting the binary.

On Linux:

```sh
sha256sum -c vogon-v0.1.0-linux-x86_64.tar.gz.sha256
tar -xzf vogon-v0.1.0-linux-x86_64.tar.gz
./vogon --version
```

On Windows PowerShell:

```powershell
$expected = (Get-Content .\vogon-v0.1.0-windows-x86_64.zip.sha256).Split()[0]
$actual = (Get-FileHash -Algorithm SHA256 .\vogon-v0.1.0-windows-x86_64.zip).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "Checksum mismatch" }
Expand-Archive .\vogon-v0.1.0-windows-x86_64.zip -DestinationPath .\vogon-release -Force
.\vogon-release\vogon.exe --version
```

Use the real version number in place of `v0.1.0`.

If the release was built by GitHub Actions, verify the archive provenance with
GitHub CLI:

```sh
gh attestation verify vogon-v0.1.0-linux-x86_64.tar.gz -R kaleab-kali/vogon-runtime
```

On Windows PowerShell:

```powershell
gh attestation verify .\vogon-v0.1.0-windows-x86_64.zip -R kaleab-kali/vogon-runtime
```

## Manual Publishing

Crate publishing to crates.io is intentionally manual while the public API is
pre-release. Before publishing, verify package contents locally and publish the
workspace crates in dependency order.
