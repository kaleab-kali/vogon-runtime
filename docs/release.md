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
python -m unittest scripts.test_write_spdx_sbom
python -m unittest scripts.test_check_benchmark_output
python -m unittest scripts.test_check_docs_links
python -m unittest scripts.test_check_env_example
python -m unittest scripts.test_check_secrets
python -m unittest scripts.test_check_workflow_policies
python scripts/check_docs_links.py --root .
python scripts/check_env_example.py --root .
python scripts/check_secrets.py --root .
python scripts/check_workflow_policies.py --root .
cargo +1.85.0 test --workspace --all-features --locked
cargo bench -p vogon-core --bench runtime -- --iterations 100 | python scripts/check_benchmark_output.py --expected-iterations 100
cargo build --release --workspace --all-features
cargo run --release -p vogon-cli -- doctor --json
cargo run --release -p vogon-cli -- init --force --output target/vogon-init-smoke/workflow.toml
cargo run --release -p vogon-cli -- check --json target/vogon-init-smoke/workflow.toml
cargo run --release -p vogon-cli -- check --json fixtures/workflows/support-triage.toml
cargo run --release -p vogon-cli -- verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo run --release -p vogon-cli -- verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
cargo run --release -p vogon-cli -- verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
cargo run --release -p vogon-cli -- trace --jsonl fixtures/replays/support-triage.replay.json
cargo install --path crates/vogon-cli --locked --offline --root target/install-smoke --force
target/install-smoke/bin/vogon --version
target/install-smoke/bin/vogon doctor --json
target/install-smoke/bin/vogon init --force --output target/install-smoke-workflow.toml
target/install-smoke/bin/vogon check --json target/install-smoke-workflow.toml
target/install-smoke/bin/vogon check --json fixtures/workflows/support-triage.toml
target/install-smoke/bin/vogon verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
target/install-smoke/bin/vogon verify fixtures/workflows/writing-pipeline.toml fixtures/replays/writing-pipeline.replay.json
docker build --tag vogon-runtime:smoke .
test "$(docker image inspect vogon-runtime:smoke --format '{{ index .Config.Labels "org.opencontainers.image.source" }}')" = "https://github.com/kaleab-kali/vogon-runtime"
test "$(docker image inspect vogon-runtime:smoke --format '{{ index .Config.Labels "org.opencontainers.image.licenses" }}')" = "MIT"
docker run --rm vogon-runtime:smoke --version
test "$(docker run --rm --entrypoint id vogon-runtime:smoke -u)" = "10001"
docker run --rm --read-only vogon-runtime:smoke --version
docker run --rm --read-only vogon-runtime:smoke doctor --json
mkdir -p target/container-smoke
chmod 777 target/container-smoke
docker run --rm --read-only -v "$PWD/target/container-smoke:/work" vogon-runtime:smoke init --force --output /work/starter.toml
docker run --rm --read-only -v "$PWD/target/container-smoke:/work:ro" vogon-runtime:smoke check --json /work/starter.toml
docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke check --json fixtures/workflows/support-triage.toml
docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
docker run --rm --read-only -v "$PWD:/work:ro" vogon-runtime:smoke trace --jsonl fixtures/replays/support-triage.replay.json
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo package --workspace --allow-dirty --no-verify --offline
```

Confirm that:

- `CHANGELOG.md` describes the release.
- `Cargo.lock` is committed.
- The GitHub Actions CI workflow has passed on `main`.
- The Security Audit workflow has a recent successful run for the committed
  `Cargo.lock`.
- If Gemini adapter or deployment behavior changed, the optional `Live Gemini
  Smoke` workflow has passed with `GEMINI_API_KEY` configured as a repository or
  environment secret.
- If OpenAI-compatible adapter or deployment behavior changed, the optional
  `Live OpenAI-Compatible Smoke` workflow has passed with
  `OPENAI_COMPATIBLE_API_KEY` configured as a repository or environment secret.
- If Groq adapter or deployment behavior changed, the optional `Live Groq
  Smoke` workflow has passed with `GROQ_API_KEY` configured as a repository or
  environment secret.
- If Hugging Face adapter or deployment behavior changed, the optional `Live
  Hugging Face Smoke` workflow has passed with `HF_TOKEN` configured as a
  repository or environment secret.
- If OpenRouter adapter or deployment behavior changed, the optional `Live
  OpenRouter Smoke` workflow has passed with `OPENROUTER_API_KEY` configured
  as a repository or environment secret.
- If container packaging changed, the container smoke commands above pass.
- The benchmark smoke command reports the expected iteration count and positive
  finite timing metrics.
- No private prompts, credentials, secrets, or sensitive replay data are present.
  The committed secret pattern scanner must pass, but maintainers should still
  review prompts and replay contents manually.
- GitHub Actions workflows keep least-privilege top-level permissions and do
  not use `pull_request_target`.

GitHub Actions workflows set `CARGO_NET_RETRY=10` for Cargo commands so
transient registry resets are retried before a job fails.

After the install smoke command, run the installed binary for your platform
against the same workflow and replay fixtures. The install smoke uses
`--offline` because earlier verification commands have already fetched and
built the locked dependencies:

```sh
target/install-smoke/bin/vogon --version
target/install-smoke/bin/vogon init --force --output target/install-smoke-workflow.toml
target/install-smoke/bin/vogon check --json target/install-smoke-workflow.toml
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
- Runs machine-readable `doctor` diagnostics against each optimized CLI
  artifact.
- Runs machine-readable trace smoke tests that assert replay schema and runtime
  metadata.
- Runs each optimized CLI artifact against every committed replay fixture.
- Packages a Linux x86_64 `vogon` binary as a `.tar.gz` archive.
- Packages a Windows x86_64 `vogon.exe` binary as a `.zip` archive.
- Builds and smoke tests the CLI container image.
- Packages the CLI container image as a `.tar.gz` archive.
- Verifies OCI source and license labels on the built and packaged container
  image.
- Includes `README.md` and `LICENSE` in each CLI archive.
- Writes `cargo metadata --locked` dependency metadata as
  `vogon-v0.1.0-cargo-metadata.json`.
- Writes an SPDX 2.3 JSON SBOM as `vogon-v0.1.0-cargo-spdx.json`.
- Writes SHA-256 checksum files for each archive.
- Writes SHA-256 checksum files for the dependency metadata and SBOM.
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

The release also publishes `vogon-v0.1.0-cargo-metadata.json`,
`vogon-v0.1.0-cargo-spdx.json`, and matching `.sha256` files. Verify them the
same way before inspecting locked dependency metadata or SBOM contents:

```sh
sha256sum -c vogon-v0.1.0-cargo-metadata.json.sha256
sha256sum -c vogon-v0.1.0-cargo-spdx.json.sha256
```

The release also publishes a container image archive and checksum:

```sh
sha256sum -c vogon-v0.1.0-container-image.tar.gz.sha256
docker load --input vogon-v0.1.0-container-image.tar.gz
test "$(docker image inspect vogon-runtime:v0.1.0 --format '{{ index .Config.Labels "org.opencontainers.image.source" }}')" = "https://github.com/kaleab-kali/vogon-runtime"
test "$(docker image inspect vogon-runtime:v0.1.0 --format '{{ index .Config.Labels "org.opencontainers.image.licenses" }}')" = "MIT"
docker run --rm vogon-runtime:v0.1.0 --version
test "$(docker run --rm --entrypoint id vogon-runtime:v0.1.0 -u)" = "10001"
docker run --rm --read-only vogon-runtime:v0.1.0 --version
docker run --rm --read-only vogon-runtime:v0.1.0 doctor --json
mkdir -p target/container-smoke
chmod 777 target/container-smoke
docker run --rm --read-only -v "$PWD/target/container-smoke:/work" vogon-runtime:v0.1.0 init --force --output /work/starter.toml
docker run --rm --read-only -v "$PWD/target/container-smoke:/work:ro" vogon-runtime:v0.1.0 check --json /work/starter.toml
```

Use the real version number in place of `v0.1.0`.

If the release was built by GitHub Actions, verify the archive provenance with
GitHub CLI:

```sh
gh attestation verify vogon-v0.1.0-linux-x86_64.tar.gz -R kaleab-kali/vogon-runtime
gh attestation verify vogon-v0.1.0-container-image.tar.gz -R kaleab-kali/vogon-runtime
```

On Windows PowerShell:

```powershell
gh attestation verify .\vogon-v0.1.0-windows-x86_64.zip -R kaleab-kali/vogon-runtime
```

## Manual Publishing

Crate publishing to crates.io is intentionally manual while the public API is
pre-release. Before publishing, verify package contents locally and publish the
workspace crates in dependency order.
