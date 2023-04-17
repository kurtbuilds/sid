set dotenv-load
set positional-arguments

run *ARGS:
    cargo run -- "$@"

test *ARGS:
    cd sid && just test

build:
    cargo build

install:
    cd cli && cargo install --path .

check:
    cargo check

alphabet:
    python3.11 perfect_hash.py