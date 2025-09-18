# Command to run
mod run './just/run.just'

# Command to build
mod build './just/build.just'

# Command to run tests
mod test './just/test.just'

# Print this list of scripts
list:
    @just --list

# Run linters and formatting
lint:
    @cargo fmt -- --check --color always
    @cargo clippy --all-targets --all-features -- -D warnings

publish-crate:
    @cargo publish
