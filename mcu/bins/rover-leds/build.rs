//! Build script to link memory.x for RP2350.

fn main() {
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
