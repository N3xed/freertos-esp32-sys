# Rust bindings and ESP32 freertos port

This library is based on [FreeRTOS-rust](https://github.com/lobaro/FreeRTOS-rust).
The bindings were generated using [bindgen](https://crates.io/crates/bindgen).

This crate currently doesn't support `esp32s2` and `esp32s3` (I don't have these two chips
on hand). Though supporting them shouldn't be that hard, it probably only requires the
right include paths and a modified `FreeRTOSConfig.h`, the esp32 FreeRTOS port in the
`port` directory should work for all three variants.

## Building

This crate uses [cc](https://github.com/alexcrichton/cc-rs) to compile the C and assembly
code. Please look at its documentation to configure compiler and archiver paths of the
xtensa gcc toolchain.

## Features

- `use-rust-alloc`  
Provide the required `vPortFree` and `pvPortMalloc` C functions using rust's global
allocator (and therefore requires the `alloc` crate to be available).

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.