mod build_freertos;

use script_utils::*;
use std::env;

// See: https://doc.rust-lang.org/cargo/reference/build-scripts.html
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let freertos_kernel_dir = format!("{}/dep/FreeRTOS-Kernel", manifest_dir);
    let esp_idf_dir = format!("{}/dep/esp-idf", manifest_dir);

    let mut b = build_freertos::Builder::new();

    // Path to FreeRTOS kernel or set ENV "FREERTOS_SRC" instead
    b.freertos_shim("port");
    b.freertos(&freertos_kernel_dir);
    b.freertos_config("port"); // Location of `FreeRTOSConfig.h`
    b.freertos_port("port/esp32".into()); // Port dir relativ to 'FreeRTOS-Kernel/portable'
                                          // b.heap("heap_4.c".into()); // Set the heap_?.c allocator to use from
                                          // 'FreeRTOS-Kernel/portable/MemMang' (Default: heap_4.c)

    b.get_cc()
        .include(&format!("{}/components/xtensa/include", esp_idf_dir))
        .include(&format!("{}/components/xtensa/esp32/include", esp_idf_dir))
        .flag("-mlongcalls");

    b.compile()
        .or_else(|e| Err(anyhow!("FreeRTOS compilation failed: {}", e.to_string())))?;

    println!(
        "cargo:rustc-link-search={}/components/xtensa/esp32",
        esp_idf_dir
    );
    println!("cargo:rustc-link-lib=xt_hal");

    Ok(())
}
