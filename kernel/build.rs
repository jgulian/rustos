pub fn main() {
    println!("cargo:rerun-if-changed=.cargo/layout.ld");
    println!("cargo:rerun-if-env-changed=VERBOSE_BUILD");
    println!("cargo:rustc-link-arg=--script=kernel/.cargo/layout.ld");
    //println!("cargo:rustc-link-arg=-L.cargo");
    println!("cargo:rustc-link-search=kernel/libs");
    println!("cargo:rustc-link-lib=sd");
}
