#[cfg(target_os = "macos")]
fn main() {
    println!("cargo:rustc-link-arg=-fapple-link-rtlib");
}