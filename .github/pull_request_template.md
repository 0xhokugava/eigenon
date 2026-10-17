## Summary

Describe the problem and the solution in a few sentences.

## Related issue

Link the issue with `Closes #123` when this pull request resolves it.

## Changes

* List the important changes.
* Keep the list focused on behavior and public interfaces.

## Validation

List the commands you ran and their results.

```text
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo test
cargo deny check advisories bans licenses sources
```

## Checklist

- [ ] The change is focused and does not mix unrelated refactors.
- [ ] Formatting passes with `cargo fmt -- --check`.
- [ ] Clippy passes with warnings denied.
- [ ] Documentation builds with rustdoc warnings denied.
- [ ] Tests pass with `cargo test`.
- [ ] Dependency policy passes with `cargo deny`.
- [ ] Behavior changes include tests when possible.
- [ ] Public behavior and commands are documented.
- [ ] Qubit-ordering conventions remain explicit where relevant.
- [ ] New or changed GitHub Actions are pinned to full commit SHAs.
- [ ] Dependency changes are intentional and `Cargo.lock` is updated.
