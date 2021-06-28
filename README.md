# Rust bindings and ESP32 freertos port

This library is based on [FreeRTOS-rust](https://github.com/lobaro/FreeRTOS-rust).
The bindings were generated using [bindgen](https://crates.io/crates/bindgen).

## Building

This crate uses [cc](https://github.com/alexcrichton/cc-rs) to compile the C and assembly
code. Please look at its documentation to configure compiler and archiver paths.

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