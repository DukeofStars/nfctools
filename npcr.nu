$env.RUSTC_WRAPPER = ""

def build [--release] {
    sweep stamp
    dx build ...(if $release {[--release]} else {[]})
}

def check [] {
    cargo check
}

def run [platform = "desktop"] {
    sweep stamp
    dx run --($platform)
}

def dev [platform = "desktop"] {
    sweep stamp
    cargo watch -c -x check -s "dx run"
}

def "test fleets" [] {
    cargo run -- --test-fleets
}

# Create a timestamp file before running cargo commands to remove all build artifacts that aren't used in them
def "sweep stamp" [] {
    cargo sweep -s
}

def sweep [--dry-run] {
    cargo sweep --file ...(if $dry_run {[--dry-run]} else {[]})
}