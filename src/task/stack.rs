use std::{os::raw::c_void, ptr::null_mut};

pub struct Stack {
    data: *mut c_void,
    size: usize,
}

impl Stack {
    pub fn bottom(&self) -> usize {
        self.data.wrapping_add(self.size) as usize
    }

    pub fn new(size: usize) -> Stack {
        assert!(size > 0);

        let mut flags = libc::MAP_ANONYMOUS | libc::MAP_PRIVATE;

        #[cfg(target_os = "linux")]
        {
            flags |= libc::MAP_STACK;
        }

        let data = unsafe {
            libc::mmap(
                null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        assert_ne!(data, null_mut(), "allocation failed");
        Stack { data, size }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe { libc::munmap(self.data, self.size) };
        }
    }
}
