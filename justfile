alias b := build
alias c := check
alias t := test
alias d := develop
alias dc := develop-client
alias dba := database-add
alias dbu := database-up
alias dbd := database-down
alias dbr := database-revert

# COMMANDS -----------------------------------------------------------------------------------------

# List commands
default:
    @just --list

# Check
check:
    cargo +nightly check && cargo +nightly clippy --all -- -W clippy::all

format:
    cargo +nightly fmt --all

# Test
test: check format
    cargo +nightly test --all && cargo +nightly sqlx prepare

# Build
build: test
    cargo +nightly build --release

# Recompile then restart the server whenever any change is made
develop:
    RUST_LOG="debug" cargo watch -q -c -w src/ -x "run"

# Re-run quick development queries whenever any change is made
develop-client:
    cargo watch -q -c -w tests/ -w src/ -x "test -q quick_dev -- --ignored --nocapture"

# Add a new database migration
database-add MIGRATION:
    sqlx migrate add -r {{ MIGRATION }}

# Initialise and/or update the database
database-up:
    sqlx database create
    sqlx migrate run

# Delete the database
database-down:
    sqlx database drop --force

# Revert a database migration
database-revert:
    sqlx migrate revert
