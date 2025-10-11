export RUST_BACKTRACE := "1"

# Builds the program
default ARGS="":
    @cargo build {{ARGS}}

# Builds the docs
[working-directory('../docs/')]
docs:
    @npm ci
    @npm run build
