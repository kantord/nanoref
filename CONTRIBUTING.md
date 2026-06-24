# Contributing to nanoref

## Prerequisites

- **Rust stable** — install via [rustup](https://rustup.rs/)
- **[pre-commit](https://github.com/pre-commit/pre-commit)** — runs lints and formatters automatically before every commit

Install pre-commit itself:

```sh
pip install pre-commit
# or: brew install pre-commit
```

## Getting started

```sh
git clone https://github.com/kantord/nanoref
cd nanoref
pre-commit install   # installs the git hook — run once after cloning
```

That's it.  After `pre-commit install`, every `git commit` automatically runs:

- `cargo fmt` — formats changed Rust files
- `cargo clippy -D warnings` — lints with zero-warning policy
- `cargo deny check` — validates dependency licenses
- standard file hygiene (trailing whitespace, TOML/YAML syntax, merge conflicts, private keys)

## Running checks manually

```sh
pre-commit run --all-files   # run all hooks against the whole tree
cargo test                   # unit + integration tests
```

## Submitting changes

Open a pull request against `main`.  CI runs the same lint and test matrix on Ubuntu and
macOS, so a clean local `pre-commit run --all-files && cargo test` is a good signal before
pushing.
