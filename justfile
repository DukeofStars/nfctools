set shell := ["nu", "-c"]

build release="":
    dx build {{release}}

full-build: build
    cargo build
    dx build --release

run logging_level="debug":
    dx run --platform

dev platform="desktop":
    dx serve --hot-patch --{{platform}}

test-fleets:
    cargo run -- --test-fleets

