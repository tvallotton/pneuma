use std::io;
use std::ops::RangeBounds;
use std::{mem::zeroed, os::raw::c_void, ptr::null_mut};

use crate::syscall;

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

    #[allow(unused_mut)]
    pub fn new(mut size: usize) -> io::Result<Stack> {
        if size == 0 {
            return unsafe { Ok(zeroed()) };
        }

        let mut flags = libc::MAP_ANONYMOUS | libc::MAP_PRIVATE;

        #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
        {
            flags |= libc::MAP_STACK;
        }

        size += page_size() - size % page_size();
        size += page_size();

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
        let stack = Stack { data, size };
        stack.protect_page()?;
        Ok(stack)
    }

    pub fn is_stackoverflow(&self, ptr: *mut c_void) -> bool {
        let range = (self.data as usize)..(self.data as usize + page_size());
        range.contains(&(ptr as usize))
    }

    pub fn protect_page(&self) -> io::Result<()> {
        if self.data.is_null() {
            return Ok(());
        }
        syscall!(mprotect, self.data as *mut _, page_size(), libc::PROT_NONE)?;

        Ok(())
    }

    pub fn stack_t(&self) -> libc::stack_t {
        libc::stack_t {
            ss_sp: self.data,
            ss_size: self.size,
            ss_flags: 0,
        }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if !self.data.is_null() {
            let _x = unsafe { libc::munmap(self.data, self.size) };
        }
    }
}
pub(crate) fn page_size() -> usize {
    thread_local! {
        static PAGE_SIZE: usize = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize};
    }
    PAGE_SIZE.with(|ps| *ps)
}
