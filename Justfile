set dotenv-load
set positional-arguments

run *ARGS:
    cargo run -- "$@"

test *ARGS:
    cargo test --all-features -- "$@"

build:
    cargo build

install:
    cd cli && cargo install --path .

check:
    cargo check
