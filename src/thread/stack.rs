use std::io;
use std::{mem::zeroed, os::raw::c_void, ptr::null_mut};



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

        let flags = libc::MAP_ANONYMOUS | libc::MAP_PRIVATE;

        #[cfg(target_os = "linux")]
        {
            flags |= libc::MAP_STACK | libc::MAP_GROWSDOWN;
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
        Ok(Stack { data, size })
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if !self.data.is_null() {
            let _x = unsafe { libc::munmap(self.data, self.size) };
        }
    }
}
