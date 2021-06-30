use chlorine::c_char;

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
