mod build_freertos;
use anyhow::*;
use std::env;

// See: https://doc.rust-lang.org/cargo/reference/build-scripts.html
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let freertos_kernel_dir = format!("{}/dep/FreeRTOS-Kernel", manifest_dir);
    let xtensa_dir = format!("{}/dep/xtensa", manifest_dir);

    let mut b = build_freertos::Builder::new();

    b.freertos_shim("port");
    b.freertos(&freertos_kernel_dir);
    b.freertos_config("port"); // Location of `FreeRTOSConfig.h`
    b.freertos_port("port/esp32".into());
    b.get_cc()
        .include(&format!("{}/include", xtensa_dir))
        .include(&format!("{}/esp32/include", xtensa_dir))
        .flag("-mlongcalls");

    b.compile()
        .or_else(|e| Err(anyhow!("FreeRTOS compilation failed: {}", e.to_string())))?;

    println!("cargo:rustc-link-search={}/esp32", xtensa_dir);
    println!("cargo:rustc-link-lib=xt_hal");

    Ok(())
}
