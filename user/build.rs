pub fn main() {
    println!("cargo:rerun-if-changed=user/.cargo/layout.ld");
    println!("cargo:rerun-if-env-changed=VERBOSE_BUILD");

    println!("cargo:rustc-link-arg=--script=user/.cargo/layout.ld");
}
