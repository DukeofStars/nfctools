$env.RUSTC_WRAPPER = ""

def build [--bundled] {
    sweep stamp
    dx build ...(if $bundled {[--features bundle]} else {[]})
}

def bundle [] {
    dx bundle --release --platform desktop --features bundle --bundle windows
}

def check [] {
    cargo check
}

def run [--bundled] {
    dx run ...(if $bundled {[--features bundle]} else {[]})
}

def dev [] {
    cargo watch -c -x check -s "dx run"
}

def serve [] {
    dx serve
}

# Create a timestamp file before running cargo commands to remove all build artifacts that aren't used in them
def "sweep stamp" [] {
    cargo sweep -s
}

def sweep [--no-build, --dry-run] {
    if not $no_build {
        # Build and generate a stamp first, then sweep
        build
    }
    cargo sweep --file ...(if $dry_run {[--dry-run]} else {[]})
}
