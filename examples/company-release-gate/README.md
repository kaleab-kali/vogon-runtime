# Company Release Gate Example

This example shows a platform team using Vogon to review a pull request that
changes deployment safety controls for a payments API. It is intentionally
small enough to audit and realistic enough to run in a separate Git
repository.

The example has two workflows:

- `workflows/context-smoke.toml` uses the deterministic adapter to prove that
  the installed CLI can collect a real Git diff, render workflow inputs, reuse
  cached output, and verify a replay. It does **not** perform useful risk
  analysis.
- `workflows/release-gate.toml` sends the diff to an explicitly allowed NVIDIA
  model, requires strict JSON decision output, writes replay evidence, and
  exits nonzero for `NO_GO`.

The candidate change disables the health check and rollback behavior, permits
zero healthy instances, and removes the request timeout. A useful model review
should block it, but model judgment is nondeterministic. Keep tests,
infrastructure policy checks, security scanning, and required human review as
separate hard gates.

This is the HTML evidence page produced by the authorized NVIDIA acceptance
run against that synthetic change:

![Company release gate report showing the blocked deployment and required remediations](../../docs/images/company-release-gate-report.png)

## Try The Offline Plumbing

Create a disposable repository and commit the baseline:

```sh
mkdir company-release-demo
cd company-release-demo
git init -b main
mkdir service
cp ../vogon-runtime/examples/company-release-gate/baseline/deployment.toml service/deployment.toml
git add service/deployment.toml
git commit -m "Add safe deployment baseline"
```

Apply the unsafe candidate:

```sh
cp ../vogon-runtime/examples/company-release-gate/candidate/deployment.toml service/deployment.toml
```

Run the installed CLI from the disposable repository:

```sh
vogon run \
  --provider deterministic \
  --git-diff \
  --repository . \
  --input service_owner=payments-platform \
  --cache-file .vogon/context.cache.json \
  --output .vogon/context.replay.json \
  ../vogon-runtime/examples/company-release-gate/workflows/context-smoke.toml
```

Repeat the command to reuse the cache, then verify the saved evidence:

```sh
vogon verify \
  --git-diff \
  --repository . \
  --input service_owner=payments-platform \
  --cache-file .vogon/context.cache.json \
  ../vogon-runtime/examples/company-release-gate/workflows/context-smoke.toml \
  .vogon/context.replay.json
```

Protect `.vogon/context.cache.json` as sensitive build data because caches hold
raw provider output. Replays contain model output and hashes, not the original
Git diff, but their contents may still be sensitive.

## Run The Actual Decision Gate

Export `NVIDIA_API_KEY` without putting it in a command argument:

```sh
vogon run \
  --provider nvidia \
  --nvidia-model meta/llama-3.1-8b-instruct \
  --git-diff \
  --repository . \
  --input service_owner=payments-platform \
  --redact-env nvidia_api_key=NVIDIA_API_KEY \
  --enforce-decision \
  --output .vogon/release-gate.replay.json \
  ../vogon-runtime/examples/company-release-gate/workflows/release-gate.toml
```

Exit code `0` means the model returned the exact allowed value `GO`. Exit code
`1` covers a denied decision and other execution failures; inspect stderr and
the replay rather than treating every nonzero result as a model denial.

Turn a written replay into a review page:

```sh
vogon report \
  --output .vogon/release-gate.html \
  .vogon/release-gate.replay.json
```

This command sends the complete tracked Git diff and the rendered service-owner
value to NVIDIA. Vogon does not currently filter diff paths or detect secrets
inside the diff. Run deterministic secret scanning first and use this gate only
where the selected provider and model satisfy the company's data-handling
policy.

The workflow allowlists the provider and model before any prompt is sent.
Changing either CLI selection fails closed. The 64 KiB output bound prevents
oversized output from entering the replay or the next prompt, but it does not
limit provider generation cost or response-body memory.

`github-actions/release-gate.yml` contains a complete pull request job that
uploads the replay even when the gate blocks the change. Add the reviewed
workflow and job to the target repository before enabling the required check:

```sh
mkdir -p .vogon .github/workflows
cp ../vogon-runtime/examples/company-release-gate/workflows/release-gate.toml .vogon/release-gate.toml
cp ../vogon-runtime/examples/company-release-gate/github-actions/release-gate.yml .github/workflows/vogon-release-gate.yml
printf '%s\n' '.vogon/*.cache.json' '.vogon/*.replay.json' '.vogon/*.html' >> .gitignore
git add .vogon/release-gate.toml .github/workflows/vogon-release-gate.yml .gitignore
```

Review the copied prompt, provider/model allowlist, service owner, artifact
retention, and pinned Vogon version as production policy. Do not enable a
third-party prompt as an unreviewed required check. GitHub does not expose
repository secrets to pull requests from forks, so this job fails closed for
fork contributions. Do not switch it to `pull_request_target` to work around
that boundary.
