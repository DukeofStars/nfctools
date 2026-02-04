build:
    #!nu
    dx build

run logging_level="debug":
    #!nu
    cargo run -- --logging-level={{logging_level}}

dev platform="desktop":
    #!nu
    dx serve --hot-patch --{{platform}}

test-fleets:
    #!nu
    cargo run -q -- --test-fleets

