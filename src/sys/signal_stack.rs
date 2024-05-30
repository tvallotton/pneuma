use std::{fmt::Write, io, mem::zeroed, ptr::null_mut};

use libc::{SA_NODEFER, SA_ONSTACK, SA_SIGINFO, SS_DISABLE};

use crate::syscall;

use super::stack::Stack;

pub struct SignalStack {
    stack: Stack,
}

impl SignalStack {
    pub fn new() -> io::Result<Self> {
        SignalStack {
            stack: Stack::new(min_sigstack_size() + 64 * 1024)?,
        }
        .install()
    }

    fn install(self) -> io::Result<Self> {
        self.install_signal_handler()?;
        self.install_alternative_stack()?;
        Ok(self)
    }

    fn install_alternative_stack(&self) -> io::Result<()> {
        let ss = self.stack.stack_t();
        syscall!(sigaltstack, &ss, null_mut())?;
        Ok(())
    }

    fn install_signal_handler(&self) -> io::Result<()> {
        let mut action: libc::sigaction = unsafe { zeroed() };
        action.sa_flags = SA_NODEFER | SA_SIGINFO | SA_ONSTACK;
        action.sa_sigaction = sigsegv_handler as usize;
        unsafe { libc::sigemptyset(&mut action.sa_mask) };

        #[cfg(target_os = "linux")]
        let signal = libc::SIGSEGV;
        #[cfg(not(target_os = "linux"))]
        let signal = libc::SIGBUS;

        syscall!(sigaction, signal, &action, null_mut())?;
        Ok(())
    }
}

impl Drop for SignalStack {
    fn drop(&mut self) {
        let mut ss = self.stack.stack_t();
        ss.ss_flags = SS_DISABLE;
        syscall!(sigaltstack, &ss, null_mut()).unwrap();
    }
}

macro_rules! raw_errln {
    ($tokens:tt) => {{
        let _ = ::core::fmt::Write::write_fmt(&mut Stderr, format_args!($tokens));
        let _ = ::core::fmt::Write::write_char(&mut Stderr, '\n');
    }};
}

fn sigsegv_handler(_signum: i32, info: &libc::siginfo_t, _data: *mut ()) {
    let rt = pneuma::runtime::current();

    let Some(lock) = rt.executor.current.get() else {
        return raw_errln!("error: segmentation fault");
    };

    let Ok(thread) = lock.try_lock() else {
        return raw_errln!("error: segmentation fault");
    };

    let name = thread.name().unwrap_or("<unknown>");
    let stack = &thread.cx.stack;

    if stack.is_stackoverflow(unsafe { info.si_addr() }) {
        raw_errln!("error: user-level thread '{name}' has overflowed its stack");
    } else {
        raw_errln!("error: segmentation fault");
    }
    std::process::abort();
}

/// Unbuffered, unsynchronized writer to stderr.
///
/// Only acceptable because everything will end soon anyways.
pub struct Stderr;

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        unsafe { libc::write(libc::STDERR_FILENO, s.as_ptr().cast(), s.len()) };
        Ok(())
    }
}

/// Modern kernels on modern hardware can have dynamic signal stack sizes.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn min_sigstack_size() -> usize {
    const AT_MINSIGSTKSZ: core::ffi::c_ulong = 51;
    let dynamic_sigstksz = unsafe { libc::getauxval(AT_MINSIGSTKSZ) };
    // If getauxval couldn't find the entry, it returns 0,
    // so take the higher of the "constant" and auxval.
    // This transparently supports older kernels which don't provide AT_MINSIGSTKSZ
    libc::MINSIGSTKSZ.max(dynamic_sigstksz as _)
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn min_sigstack_size() -> usize {
    libc::MINSIGSTKSZ
}
