export RUST_BACKTRACE := "1"

# Print this list of scripts
list:
    @just --list build 

# Builds the program for Windows targets 
windows ARGS="":
    @cargo build --target="x86_64-pc-windows-msvc" {{ARGS}}

# Builds the program for Linux targets
linux ARGS="":
    @cargo build --target="x86_64-unknown-linux-gnu" {{ARGS}}

# Builds the program for Macos targets
macos ARGS="":
    @cargo build --target="x86_64-apple-darwin" {{ARGS}}

# Builds the docs
[working-directory('../docs/')]
docs:
    @npm ci
    @npm run build
