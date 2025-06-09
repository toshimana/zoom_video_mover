fn main() {
    // Build script simplified to avoid Windows API linking issues
    println!("cargo:rerun-if-changed=build.rs");
}