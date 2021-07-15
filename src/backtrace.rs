//! Backtrace support for printing stack pointers and program counters.

use core::ffi::c_void;

/// A frame in the backtrace
#[derive(Debug)]
pub struct BacktraceFrame {
    /// The address of the last instruction in the backtrace (program counter).
    pub pc: u32,
    /// The address of the current stack frame in the backtrace (stack pointer).
    pub sp: u32,
}

impl core::fmt::Display for BacktraceFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#08x}:{:#08x}", self.pc, self.sp)
    }
}

impl BacktraceFrame {
    /// Check if `pc` and `sp` are sane.
    ///
    /// Checks if the stack pointer is located in dram (data ram) and if the program
    /// counter is in an executable memory space.
    ///
    /// Taken from `esp-idf/components/xtensa/debug_helpers.c
    pub fn is_sane(&self) -> bool {
        let sp_in_dram = {
            let sp = self.sp as usize;
            !(sp < (SOC_DRAM_LOW + 0x10) || sp > (SOC_DRAM_HIGH - 0x10))
        };

        sp_in_dram// && is_pointer_executable(self.pc as usize)
    }
}

/// Wether or not the supplied address is in an executable memory space.
pub fn is_pointer_executable(ptr: usize) -> bool {
    (ptr >= SOC_IROM_LOW && ptr < SOC_IROM_HIGH)
        || (ptr >= SOC_IRAM_LOW && ptr < SOC_IRAM_HIGH)
        || (ptr >= SOC_IROM_MASK_LOW && ptr < SOC_IROM_MASK_HIGH)
        || (ptr >= SOC_CACHE_APP_LOW && ptr < SOC_CACHE_APP_HIGH)
        || (ptr >= SOC_CACHE_PRO_LOW && ptr < SOC_CACHE_PRO_HIGH)
        || (ptr >= SOC_RTC_IRAM_LOW && ptr < SOC_RTC_IRAM_HIGH)
}

// Memory map
// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`

/// Data ROM starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_DROM_LOW: usize = 0x3F400000;
/// Data ROM ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_DROM_HIGH: usize = 0x3F800000;
/// Data RAM starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_DRAM_LOW: usize = 0x3FFAE000;
/// Data RAM ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_DRAM_HIGH: usize = 0x40000000;
/// Instruction ROM starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_IROM_LOW: usize = 0x400D0000;
/// Instruction ROM ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_IROM_HIGH: usize = 0x40400000;
/// Instruction ROM mask starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_IROM_MASK_LOW: usize = 0x40000000;
/// Instruction ROM mask ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_IROM_MASK_HIGH: usize = 0x40064F00;
/// Pro cpu cache starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_CACHE_PRO_LOW: usize = 0x40070000;
/// Pro cpu cache ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_CACHE_PRO_HIGH: usize = 0x40078000;
/// App cpu cache starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_CACHE_APP_LOW: usize = 0x40078000;
/// App cpu cache ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_CACHE_APP_HIGH: usize = 0x40080000;
/// Instruction RAM starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_IRAM_LOW: usize = 0x40080000;
/// Instruction RAM ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_IRAM_HIGH: usize = 0x400A0000;
/// RTC instruction RAM starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_RTC_IRAM_LOW: usize = 0x400C0000;
/// RTC instruction RAM ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_RTC_IRAM_HIGH: usize = 0x400C2000;
/// Rtc data ram starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_RTC_DRAM_LOW: usize = 0x3FF80000;
/// Rtc data ram ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_RTC_DRAM_HIGH: usize = 0x3FF82000;
/// Rtc data starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_RTC_DATA_LOW: usize = 0x50000000;
/// Rtc data ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_RTC_DATA_HIGH: usize = 0x50002000;
/// External RAM starting address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_EXTRAM_DATA_LOW: usize = 0x3F800000;
/// External RAM ending address
/// Taken from `esp-idf/components/soc/soc/esp32/include/soc/soc.h`
pub const SOC_EXTRAM_DATA_HIGH: usize = 0x3FC00000;

/// A backtrace iterator that returns [`BacktraceFrame`]s.
pub struct Backtrace {
    pc: u32,
    sp: u32,
    next_pc: u32,
    last: bool,
}

impl Backtrace {
    /// Create a new backtrace
    ///
    /// Given the following function call flow (B -> A -> Backtrace::new)
    /// this function will do the following.
    /// - Flush CPU registers and window frames onto the current stack
    /// - Setup PC and SP of function A (i.e. start of the stack's backtrace)
    /// - Setup PC of function B in `next_pc`
    pub fn new() -> Backtrace {
        let mut frame = Backtrace {
            pc: 0,
            sp: 0,
            next_pc: 0,
            last: false,
        };
        unsafe {
            super::esp_backtrace_get_start(
                &mut frame.pc as *mut _,
                &mut frame.sp as *mut _,
                &mut frame.next_pc as *mut _,
            );
        }

        frame
    }

    /// Convert the PC register value to its true address
    ///
    /// The address of the current instruction is not stored as an exact u32
    /// representation in PC register. This function will convert the value stored in the
    /// PC register to a u32 address.
    ///
    /// Ported from `esp-idf/components/soc/include/soc/cpu.h`
    pub fn get_real_pc(&self) -> u32 {
        let mut pc = self.pc;
        if (pc & 0x80000000) > 0 {
            pc = (pc & 0x3fffffff) | 0x40000000;
        }
        pc.saturating_sub(3)
    }

    /// Get the previous stack frame from the current stack pointer
    pub fn next_stack_frame(&mut self) {
        // Use frame(i-1)'s BS area located below frame(i)'s sp to get frame(i-1)'s sp and frame(i-2)'s pc
        unsafe {
            let base_save = self.sp as *const c_void; // Base save area consists of 4 words under SP
            self.pc = self.next_pc;
            self.next_pc = *(base_save.sub(16) as *const u32); //If next_pc = 0, indicates frame(i-1) is the last frame on the stack
            self.sp = *(base_save.sub(12) as *const u32);
        }
    }
}

impl core::iter::Iterator for Backtrace {
    type Item = BacktraceFrame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last {
            return None;
        }

        let res = BacktraceFrame {
            pc: self.get_real_pc(),
            sp: self.sp,
        };

        if res.is_sane() {
            self.next_stack_frame();
        } else {
            self.last = true;
        }

        Some(res)
    }
}
