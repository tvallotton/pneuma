use std::{fmt::Write, io, mem::zeroed, ptr::null_mut};

use libc::{SA_NODEFER, SA_ONSTACK, SA_SIGINFO, SS_DISABLE};

use crate::syscall;

use super::stack::Stack;
use std::ptr;

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
        raw_errln!("error: segmentation fault");
        return print_stack_trace();
    };

    let name = thread.name().unwrap_or("<unknown>");
    let stack = &thread.cx.stack;

    if stack.is_stackoverflow(unsafe { info.si_addr() }) {
        raw_errln!("error: user-level thread '{name}' has overflowed its stack");
        print_stack_trace();
    } else {
        raw_errln!("error: segmentation fault");
        print_stack_trace();
    }
}

extern "C" {
    fn backtrace_symbols_fd(buffer: *const *mut libc::c_void, size: libc::c_int, fd: libc::c_int);
}

fn backtrace_stderr(buffer: &[*mut libc::c_void]) {
    let size = buffer.len().try_into().unwrap_or_default();
    unsafe { backtrace_symbols_fd(buffer.as_ptr(), size, libc::STDERR_FILENO) };
}
/// Unbuffered, unsynchronized writer to stderr.
///
/// Only acceptable because everything will end soon anyways.
pub struct Stderr;

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        unsafe { libc::write(2, s.as_ptr().cast(), s.len()) };
        Ok(())
    }
}

/// Signal handler installed for SIGSEGV
extern "C" fn print_stack_trace() {
    const MAX_FRAMES: usize = 256;
    // Reserve data segment so we don't have to malloc in a signal handler, which might fail
    // in incredibly undesirable and unexpected ways due to e.g. the allocator deadlocking
    static mut STACK_TRACE: [*mut libc::c_void; MAX_FRAMES] = [ptr::null_mut(); MAX_FRAMES];
    let stack = unsafe {
        // Collect return addresses
        let depth = libc::backtrace(STACK_TRACE.as_mut_ptr(), MAX_FRAMES as i32);
        if depth == 0 {
            return;
        }
        &STACK_TRACE.as_slice()[0..(depth as _)]
    };

    // Just a stack trace is cryptic. Explain what we're doing.
    raw_errln!("error: printing backtrace\n");
    let mut written = 1;
    let mut consumed = 0;
    // Begin elaborating return addrs into symbols and writing them directly to stderr
    // Most backtraces are stack overflow, most stack overflows are from recursion
    // Check for cycles before writing 250 lines of the same ~5 symbols
    let cycled = |(runner, walker)| runner == walker;
    if let Some(period) = stack.iter().skip(1).step_by(2).zip(stack).position(cycled) {
        let period = period.saturating_add(1); // avoid "what if wrapped?" branches
        let Some(offset) = stack.iter().skip(period).zip(stack).position(cycled) else {
            // impossible.
            return;
        };

        // Count matching trace slices, else we could miscount "biphasic cycles"
        // with the same period + loop entry but a different inner loop
        let next_cycle = stack[offset..].chunks_exact(period).skip(1);
        let cycles = 1 + next_cycle
            .zip(stack[offset..].chunks_exact(period))
            .filter(|(next, prev)| next == prev)
            .count();
        backtrace_stderr(&stack[..offset]);
        written += offset;
        consumed += offset;
        if cycles > 1 {
            raw_errln!("\n### cycle encountered after {offset} frames with period {period}");
            backtrace_stderr(&stack[consumed..consumed + period]);
            raw_errln!("### recursed {cycles} times\n");
            written += period + 4;
            consumed += period * cycles;
        };
    }
    let rem = &stack[consumed..];
    backtrace_stderr(rem);
    raw_errln!("");
    written += rem.len() + 1;

    if stack.len() == MAX_FRAMES {
        raw_errln!("note: maximum backtrace depth reached, frames may have been lost");
        written += 1;
    }

    written += 2;
    if written > 24 {
        // We probably just scrolled the earlier "we got SIGSEGV" message off the terminal
        raw_errln!("note: backtrace dumped due to SIGSEGV! resuming signal");
    };
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
