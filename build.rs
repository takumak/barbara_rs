use cc::Build;

fn main() {
    Build::new().file("src/reset.s").compile("asm");
    println!("cargo:rerun-if-changed=src/reset.s");
    println!("cargo:rerun-if-changed=memory.x");
}
