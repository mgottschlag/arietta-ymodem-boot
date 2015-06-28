
use core::fmt;

#[lang="stack_exhausted"]
extern fn stack_exhausted() {}

#[lang="eh_personality"]
extern fn eh_personality() {}

#[lang="begin_unwind"]
extern fn begin_unwind() {}

#[no_stack_check]
#[no_mangle]
pub extern fn abort() -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, num: isize) {
    for i in 1..num {
        unsafe {
            *dest.offset(i) = *src.offset(i)
        }
    }
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_unwind(msg: fmt::Arguments,
                                file: &'static str, line: u32) -> ! {
    loop {}
}

