# Deployment

Vogon Runtime ships as a Rust CLI. The release workflow publishes native Linux
and Windows binaries, and the repository includes a Dockerfile for environments
that standardize on container images.

## Build a Container Image

Build the CLI image from the repository root:

```sh
docker build --tag vogon-runtime:local .
```

The image uses the `vogon` binary as its entrypoint, so CLI arguments are passed
directly:

```sh
docker run --rm vogon-runtime:local --version
```

Mount the repository, or a directory containing workflows and replays, at
`/work` to run against local files:

```sh
docker run --rm -v "$PWD:/work" vogon-runtime:local check fixtures/workflows/support-triage.toml
docker run --rm -v "$PWD:/work" vogon-runtime:local verify fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
```

## Provider Credentials

The deterministic provider does not require network access or credentials. For
provider-backed runs, pass credentials as environment variables at runtime:

```sh
docker run --rm \
  -e GEMINI_API_KEY \
  -v "$PWD:/work" \
  vogon-runtime:local run --provider gemini fixtures/workflows/support-triage.toml
```

For OpenAI-compatible endpoints:

```sh
docker run --rm \
  -e OPENAI_COMPATIBLE_API_KEY \
  -v "$PWD:/work" \
  vogon-runtime:local run --provider openai-compatible fixtures/workflows/support-triage.toml
```

Use `--openai-compatible-base-url` and `--openai-compatible-model` to target a
specific compatible service.

## Runtime Notes

- The runtime image is based on Debian bookworm slim.
- CA certificates are installed so HTTPS provider calls work by default.
- The container runs as the unprivileged `vogon` user.
- Workflow and replay files should be mounted into `/work` instead of baked
  into the image.

## Smoke Test

Before publishing or deploying an image, run:

```sh
docker build --tag vogon-runtime:smoke .
docker run --rm vogon-runtime:smoke --version
docker run --rm -v "$PWD:/work" vogon-runtime:smoke check --json fixtures/workflows/support-triage.toml
docker run --rm -v "$PWD:/work" vogon-runtime:smoke verify --json fixtures/workflows/support-triage.toml fixtures/replays/support-triage.replay.json
docker run --rm -v "$PWD:/work" vogon-runtime:smoke trace --jsonl fixtures/replays/support-triage.replay.json
```
