fn main() {
    println!("cargo::rerun-if-changed=ui/");
    slint_build::compile("ui/main-window.slint").expect("Failed to build ui");
}
