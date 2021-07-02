#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(feature = "use-rust-alloc")]
extern crate alloc;

pub mod backtrace;
mod bindings;
pub mod glue;

pub use bindings::*;

// TODO: Bindgen should also generate these
pub const pdTRUE: BaseType_t = 1;
pub const pdFALSE: BaseType_t = 0;
pub const pdPASS: BaseType_t = pdTRUE;
pub const pdFAIL: BaseType_t = pdFALSE;

pub const queueQUEUE_TYPE_BASE: u8 = 0;
pub const queueQUEUE_TYPE_SET: u8 = 0;
pub const queueQUEUE_TYPE_MUTEX: u8 = 1;
pub const queueQUEUE_TYPE_COUNTING_SEMAPHORE: u8 = 2;
pub const queueQUEUE_TYPE_BINARY_SEMAPHORE: u8 = 3;
pub const queueQUEUE_TYPE_RECURSIVE_MUTEX: u8 = 4;
