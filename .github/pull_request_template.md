## Summary

-

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo build --release --workspace --all-features`
- [ ] `cargo doc --workspace --all-features --no-deps`
- [ ] `cargo package --workspace --allow-dirty --no-verify`
- [ ] Relevant CLI smoke test:

## Checklist

- [ ] The change is focused and reviewable.
- [ ] Tests were added or updated for behavior changes.
- [ ] Public docs were updated for user-facing behavior changes.
- [ ] Provider-specific code remains outside `vogon-core`.
- [ ] No secrets, credentials, private prompts, or sensitive replay data are included.

## Notes

-
