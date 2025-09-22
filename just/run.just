export RUST_BACKTRACE := "1"

# Print this list of scripts
list:
    @just --list run

# Runs the program in release mode
release ARGS="": 
    @cargo run --release {{ARGS}}

# Runs the program in development mode 
bin ARGS="":
    @cargo run {{ARGS}}

# Serves the docs in development-mode
[working-directory("../docs/")]
docs:
    @npm install
    @npm start
