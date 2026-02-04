build:
    #!nu
    dx build

dev platform="desktop":
    #!nu
    dx serve --hot-patch --{{platform}}

test-fleets:
    #!nu
    cargo run -q -- --test-fleets