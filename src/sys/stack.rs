use std::{
    io::{self, Error},
    os::raw::c_void,
    ptr::null_mut,
};

use pneuma::syscall;

#[repr(C)]
pub(crate) struct Stack {
    data: *mut c_void,
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
            return Ok(Stack::default());
        }

        size += page_size() - size % page_size();
        size += page_size();

        let data = syscall!(
            mmap,
            null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE | map_stack(),
            -1,
            0,
        )?;

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

    #[allow(unused_mut)]
    pub fn try_grow(&mut self) -> io::Result<()> {
        let target_ptr = self.data.wrapping_sub(page_size());

        let data = syscall!(
            mmap,
            target_ptr,
            page_size(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE | map_fixed() | map_stack(),
            -1,
            0,
        )?;

        if data != target_ptr {
            return Err(Error::other("failed to allocate specified mapping"));
        }

        self.data = data;
        self.size += page_size();
        self.protect_page()?;

        Ok(())
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if !self.data.is_null() {
            let _x = unsafe { libc::munmap(self.data, self.size) };
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Stack {
            data: null_mut(),
            size: 0,
        }
    }
}

pub(crate) fn page_size() -> usize {
    thread_local! {
        static PAGE_SIZE: usize = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize};
    }
    PAGE_SIZE.with(|ps| *ps)
}

unsafe impl Sync for Stack {}
unsafe impl Send for Stack {}

#[allow(unreachable_code)]
fn map_stack() -> i32 {
    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
    {
        return libc::MAP_STACK;
    };
    0
}

#[allow(unreachable_code)]
fn map_fixed() -> i32 {
    #[cfg(target_os = "linux")]
    {
        return libc::MAP_FIXED_NOREPLACE;
    };
    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "netbsd"
    ))]
    {
        return libc::MAP_STACK;
    };
    0
}
