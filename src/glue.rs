use core::{
    sync::atomic::{AtomicI32, Ordering},
};

use chlorine::c_char;
use spin::mutex::{SpinMutex, SpinMutexGuard};

use crate::{vTaskEnterCritical, vTaskExitCritical, UBaseType_t};

#[cfg(feature = "use-rust-alloc")]
mod malloc_impl {
    use core::alloc::Layout;
    use core::mem;

    use chlorine::c_void;

    #[no_mangle]
    unsafe extern "C" fn vPortFree(ptr: *mut c_void) {
        if ptr != core::ptr::null_mut() {
            let ptr = ptr.sub(mem::size_of::<usize>());
            // get block size
            let size: usize = *mem::transmute::<_, *mut usize>(ptr);

            ::alloc::alloc::dealloc(ptr as *mut u8, Layout::from_size_align_unchecked(size, 4));
        }
    }

    #[no_mangle]
    unsafe extern "C" fn pvPortMalloc(wanted_size: usize) -> *mut c_void {
        if wanted_size == 0 {
            return core::ptr::null_mut();
        }

        let layout = Layout::from_size_align(wanted_size + mem::size_of::<usize>(), 4).unwrap();

        let ptr = ::alloc::alloc::alloc(layout);
        if ptr == core::ptr::null_mut() {
            return core::ptr::null_mut();
        }

        // set first usize to `layout.size()`
        let ptr_usize: *mut usize = mem::transmute(ptr);
        *ptr_usize = layout.size();

        mem::transmute(ptr.add(mem::size_of::<usize>()))
    }
}

#[no_mangle]
unsafe extern "C" fn vPortPanic(
    file: *const c_char,
    file_len: usize,
    line: usize,
    func: *const c_char,
    func_len: usize,
) {
    let file = if file_len == 0 {
        ""
    } else {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(file as *const u8, file_len))
    };
    let func = if func_len == 0 {
        ""
    } else {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(func as *const u8, func_len))
    };
    panic!("{}:{} ({}) - RTOS assert failed!", file, line, func);
}

#[no_mangle]
unsafe extern "C" fn vApplicationStackOverflowHook(
    _task: super::TaskHandle_t,
    task_name: *mut c_char,
) {
    let len = super::strlen(task_name);
    let task_name = if len == 0 {
        ""
    } else {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            task_name as *const u8,
            len as usize,
        ))
    };

    panic!("StackOverflow in task '{}'", task_name);
}

#[inline]
pub unsafe fn vPortYieldFromISR() {
    crate::_frxt_setup_switch()
}

#[inline]
pub unsafe fn ulTaskEnterCriticalFromISR() -> UBaseType_t {
    let mut state: UBaseType_t;
    llvm_asm!("rsr.ps $0" : "=a"(state) :: "memory");
    vTaskEnterCritical();

    state
}

#[inline]
pub unsafe fn vTaskExitCriticalFromISR(previous_state: UBaseType_t) {
    vTaskExitCritical();
    llvm_asm!("wsr.ps $0" :: "r"(previous_state) : "memory")
}

#[inline]
pub fn portGET_CORE_ID() -> UBaseType_t {
    let mut id: u32;
    unsafe {
        llvm_asm!(r#"rsr.prid $0;
              extui $0, $0, 13, 1"# : "=r"(id));
    }
    id
}

static ISR_LOCK: (SpinMutex<usize>, AtomicI32) = (SpinMutex::new(0), AtomicI32::new(-1));
static TASK_LOCK: (SpinMutex<usize>, AtomicI32) = (SpinMutex::new(0), AtomicI32::new(-1));

#[link_section = ".rwtext"]
pub fn take_lock_recursive((lock, owner): &(SpinMutex<usize>, AtomicI32)) {
    let core_id = portGET_CORE_ID() as i32;

    let w = if owner.load(Ordering::Relaxed) == core_id {
        // Safe because we checked that this core already owns the lock.
        let w = unsafe { &mut *lock.as_mut_ptr() };

        w
    } else {
        // Wait for the other core to unlock the mutex.
        let l = lock.lock();
        owner.store(core_id, Ordering::Relaxed);

        // Keep the mutex locked, it will be unlocked in `give_lock_recursive`.
        SpinMutexGuard::leak(l)
    };

    // Increment the lock count.
    *w += 1;
}

#[link_section = ".rwtext"]
pub fn give_lock_recursive((lock, owner): &(SpinMutex<usize>, AtomicI32)) {
    let core_id = portGET_CORE_ID() as i32;

    // The current core should always own the lock on a call to this function.
    debug_assert!(owner.load(Ordering::Relaxed) == core_id);

    // Safe because this core owns the lock (see assert above).
    let w = unsafe { &mut *lock.as_mut_ptr() };
    let count = *w - 1;
    *w = count;

    // Unlock the mutex because the count is zero.
    if count == 0 {
        // `-1` signals an unowned lock.
        // Note this MUST happen before the lock is unlocked, otherwise it causes a race
        // condition in `take_lock_recursive()`.
        owner.store(-1, Ordering::Relaxed);

        // Safe because this core owns the lock and the lock count is 0.
        unsafe {
            lock.force_unlock();
        }
    }
}

#[no_mangle]
#[link_section = ".rwtext"]
pub extern "C" fn vPortTakeISRLock() {
    take_lock_recursive(&ISR_LOCK);
}

#[no_mangle]
#[link_section = ".rwtext"]
pub extern "C" fn vPortGiveISRLock() {
    give_lock_recursive(&ISR_LOCK);
}

#[no_mangle]
#[link_section = ".rwtext"]
pub extern "C" fn vPortTakeTaskLock() {
    take_lock_recursive(&TASK_LOCK);
}

#[no_mangle]
#[link_section = ".rwtext"]
pub extern "C" fn vPortGiveTaskLock() {
    give_lock_recursive(&TASK_LOCK);
}
