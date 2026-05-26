# Contributing

Thanks for helping improve Vogon Runtime.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Pull Requests

- Keep changes focused and reviewable.
- Add or update tests for behavior changes.
- Update docs when user-facing behavior changes.
- Keep provider-specific code outside `vogon-core`.
- Do not commit secrets, credentials, or real customer data.

## Commit Style

Use short imperative commit messages, for example:

```text
Add workspace scaffold
Implement deterministic echo adapter
Document replay format
```
