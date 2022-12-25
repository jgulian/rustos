pub fn main() {
    println!("cargo:rerun-if-changed=config.toml/layout.ld");
}
