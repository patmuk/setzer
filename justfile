set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Lints, formatting, tests
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
