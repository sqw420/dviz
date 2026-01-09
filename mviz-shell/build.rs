fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "macos" {
        cc::Build::new()
            .file("src/metal_xpc.m")
            .compile("metal_xpc");

        println!("cargo:rerun-if-changed=src/metal_xpc.m");
    }
}
