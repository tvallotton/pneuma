use std::io;
use std::{alloc::Layout, mem::zeroed, os::raw::c_void, ptr::null_mut};

use std::alloc::alloc;

use libc::PACKET_ADD_MEMBERSHIP;

thread_local! {
    static PAGE_SIZE: usize = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize};
}

#[repr(C)]
pub(crate) struct Stack {
    pub data: *mut c_void,
    pub size: usize,
}

impl Stack {
    pub fn bottom(&self) -> u64 {
        let out = (self.data as u64) + self.size as u64 / 2;
        assert_eq!(out % 16, 0);
        out
    }

    pub fn new(mut size: usize) -> io::Result<Stack> {
        if size == 0 {
            return unsafe { zeroed() };
        }

        let mut flags = libc::MAP_ANONYMOUS | libc::MAP_PRIVATE | libc::MAP_GROWSDOWN;

        #[cfg(target_os = "linux")]
        {
            flags |= libc::MAP_STACK;
        }

        let page_size = PAGE_SIZE.with(|s| *s);
        size += page_size - size % page_size;
        let data = unsafe {
            libc::mmap(
                null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                flags,
                -1,
                0,
            )
        };

        if data as i64 == -1 {
            return Err(io::Error::last_os_error());
        }
        let data = unsafe { alloc(Layout::array::<u8>(size).unwrap()).cast() };
        Ok(Stack { data, size })
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if !self.data.is_null() {
            // unsafe {
            //     std::alloc::dealloc(self.data.cast(), Layout::array::<u8>(self.size).unwrap())
            // }
            unsafe {
                std::arch::asm!("udf #0");
            }
            unsafe { libc::munmap(self.data, self.size) };
        }
    }
}
