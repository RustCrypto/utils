extern crate vgrs;

use vgrs::memcheck::{make_mem_undefined, make_mem_defined};

pub fn poison(addr: *const (), len: usize) {
    unsafe {
        make_mem_undefined(addr, len);
    }
}

pub fn unpoison(addr: *const (), len: usize) {
    unsafe {
        make_mem_defined(addr, len);
    }
}
