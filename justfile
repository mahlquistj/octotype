# Command to run
mod run './just/run.just'

# Command to build
mod build './just/build.just'

# Print this list of scripts
list:
    @just --list

# Run linters and formatting
lint:
    @cargo fmt -- --check --color always
    @cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test ARGS="":
    @cargo nextest run --hide-progress-bar --failure-output final {{ARGS}}

publish:
    @cargo publish
