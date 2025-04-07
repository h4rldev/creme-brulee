set quiet

default:
    just --list

build-release:
    cargo build --release

run-release:
    ./target/release/creme-brulee

build:
    cargo build

run:
    cargo run

test:
    cargo test

clippy:
    cargo clippy

fmt:
    cargo fmt

clean:
    cargo clean
