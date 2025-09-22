export RUST_BACKTRACE := "1"

# Print this list of scripts
list:
    @just --list test

# Runs tests for windows
windows ARGS="":
    @cargo nextest run --target="x86_64-pc-windows-msvc" \
        --hide-progress-bar --failure-output final {{ARGS}}

# Runs tests for linux
linux ARGS="":
    @cargo nextest run --target="x86_64-unknown-linux-gnu" \
        --hide-progress-bar --failure-output final {{ARGS}}

# Runs tests for macos
macos ARGS="":
    @cargo nextest run --target="x86_64-apple-darwin" \
        --hide-progress-bar --failure-output final {{ARGS}}

