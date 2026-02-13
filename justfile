set shell := ["nu", "-c"]

build release="":
    dx build {{release}}

full-build: build
    cargo build
    dx build --release

run logging_level="debug":
    cargo run -- --logging-level={{logging_level}}

dev platform="desktop":
    dx serve --hot-patch --{{platform}}

test-fleets:
    cargo run -- --test-fleets

