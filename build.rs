fn main() {
    // Set our custom linker script
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}
