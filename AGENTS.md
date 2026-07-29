# Repository Agent Instructions

These instructions apply to the entire repository.

## Public Project Standards

- Keep the README public-facing. It should explain what Vogon Runtime is, why it
  exists, how to use it, and how to contribute.
- Do not add internal planning notes, private checklists, portfolio notes, or
  session-specific implementation scratchpads to public docs.
- Keep the repository open-source ready: clear docs, small changes, tests, CI,
  license clarity, and reviewable pull requests.
- Keep Vogon Runtime Rust-first. Product code, long-lived project tooling, and
  new validators should default to Rust, preferably through the workspace or a
  Rust `xtask` tool. Do not add new Python validators by default.
- Treat the existing Python scripts as transitional CI/release tooling. When
  improving checks, prefer consolidating or migrating them into Rust instead of
  growing the Python surface area.
- Use actual provider APIs and realistic responses for claimed live-provider
  smoke or acceptance evidence when the user has approved credential use. Do
  not present a fake server or simulated model response as live-provider proof.
- Keep deterministic adapters for unit tests, CI, fixtures, and offline
  development so automated verification remains reproducible.
- Never print, commit, serialize, or otherwise expose provider API keys.

## Git Workflow

- Use functional branch names that describe the change, for example
  `scaffold-rust-workspace`, `parse-workflow-files`, or
  `verify-replay-logs`.
- Do not use phase-based branch names, PR titles, or commit messages.
- Keep commits focused and use concise imperative commit messages.
- Use pull requests for changes to `main`.
- Merge pull requests with regular merge commits only.
- Do not squash merge.
- Do not delete local or remote branches unless the user explicitly requests
  branch deletion.

## Commit Metadata

- Never add AI co-author trailers.
- Never add `Co-authored-by` trailers for Claude, Codex, OpenAI, or other AI
  tools.
- Commits should use the repository owner's configured Git identity unless the
  user explicitly requests otherwise.

## Testing Expectations

- Run pre-change or pre-merge checks before merging a stage when practical.
- Run post-change checks after each functional stage.
- At minimum, use:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

- Also run any relevant CLI smoke test for the feature being changed.
- Include the verification commands in every PR body.

## Change Size

- Work in small functional pieces.
- Each PR should be understandable on its own.
- Prefer the existing crate boundaries:
  - `vogon-core` for provider-neutral runtime logic.
  - `vogon-adapters` for model adapter implementations.
  - `vogon-cli` for command-line behavior.
