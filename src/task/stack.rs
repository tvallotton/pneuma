use std::{alloc::Layout, mem::zeroed, os::raw::c_void, ptr::null_mut};

use std::alloc::alloc;

#[repr(C)]
pub struct Stack {
    pub data: *mut c_void,
    pub size: usize,
}

impl Stack {
    pub fn bottom(&self) -> u64 {
        (self.data as u64) + (self.size as u64 / 2)
    }

    pub fn new(size: usize) -> Stack {
        if size == 0 {
            return unsafe { zeroed() };
        }

        // let mut flags = libc::MAP_ANONYMOUS | libc::MAP_PRIVATE;

        // #[cfg(target_os = "linux")]
        // {
        //     flags |= libc::MAP_STACK;
        // }

        // let data = unsafe {
        //     libc::mmap(
        //         null_mut(),
        //         size,
        //         libc::PROT_READ | libc::PROT_WRITE,
        //         libc::MAP_ANONYMOUS,
        //         -1,
        //         0,
        //     )
        // };
        let data = unsafe { alloc(Layout::array::<u8>(size).unwrap()).cast() };
        assert_ne!(data, null_mut(), "allocation failed");
        Stack { data, size }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                std::alloc::dealloc(self.data.cast(), Layout::array::<u8>(self.size).unwrap())
            }
            // unsafe { libc::munmap(self.data, self.size) };
        }
    }
}
