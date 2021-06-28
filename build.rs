mod build_freertos;

use script_utils::*;
use std::env;
use std::fs;
use std::path::Path;

// See: https://doc.rust-lang.org/cargo/reference/build-scripts.html
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let freertos_kernel_dir = format!("{}/dep/FreeRTOS-Kernel", manifest_dir);
    let esp_idf_dir = format!("{}/dep/esp-idf", manifest_dir);

    if !Path::new(&freertos_kernel_dir).exists() {
        println!("cargo:warning=FreeRTOS-Kernel not found!");
        println!(
            "cargo:warning=Cloning 'https://github.com/FreeRTOS/FreeRTOS-Kernel.git' into '{}'",
            out_dir
        );

        fs::create_dir_all(&freertos_kernel_dir)?;
        (cmd_lib::run_cmd! {
            cd "${freertos_kernel_dir}";
            git init;
            git remote add -f origin "https://github.com/FreeRTOS/FreeRTOS-Kernel.git";
            git sparse-checkout set "/include" "portable/Common" "/*.*";
            git sparse-checkout init;
            git fetch origin main --depth 1;
            git checkout main;
        })?;
    }

    if !Path::new(&esp_idf_dir).exists() {
        fs::create_dir_all(&esp_idf_dir)?;
        (cmd_lib::run_cmd! {
            cd "${esp_idf_dir}";
            git init;
            git remote add -f origin "https://github.com/espressif/esp-idf.git";
            git sparse-checkout set "/components/xtensa";
            git sparse-checkout init;
            git fetch origin 1d7068e4be430edd92bb63f2d922036dcf5c3cc1 --depth 1;
            git checkout 1d7068e4be430edd92bb63f2d922036dcf5c3cc1;
        })?;
    }

    let mut b = build_freertos::Builder::new();

    // Path to FreeRTOS kernel or set ENV "FREERTOS_SRC" instead
    b.freertos_shim("port");
    b.freertos(&freertos_kernel_dir);
    b.freertos_config("port"); // Location of `FreeRTOSConfig.h`
    b.freertos_port("port/esp32".into()); // Port dir relativ to 'FreeRTOS-Kernel/portable'
    // b.heap("heap_4.c".into()); // Set the heap_?.c allocator to use from
                                         // 'FreeRTOS-Kernel/portable/MemMang' (Default: heap_4.c)

    b.get_cc()
        .compiler(&format!(
            "{}/../../tools/gcc/bin/xtensa-esp32-elf-gcc.exe",
            manifest_dir
        ))
        .archiver(&format!(
            "{}/../../tools/gcc/bin/xtensa-esp32-elf-ar.exe",
            manifest_dir
        ))
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
