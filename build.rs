use std::path::Path;

use base64::prelude::*;

fn main() {
    // Encode font into base64 to be loaded by CSS
    println!("cargo::rerun-if-changed=assets/bombardier.ttf");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let font_bytes = std::fs::read("assets/bombardier.ttf").unwrap();
    let encoded = BASE64_STANDARD.encode(font_bytes);

    std::fs::write(Path::new(&out_dir).join("bombardier.ttf.base64"), encoded)
        .unwrap();
}
