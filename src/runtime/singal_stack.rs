use crate::{syscall, thread::Stack};

use std::{io, mem::zeroed, ptr::null_mut};

use libc::{SA_ONSTACK, SA_SIGINFO, SS_DISABLE};

/// This struct when initialized will set itself to be the
/// signal handler stack for the current thread. When dropped,
/// it will remove itself.
pub struct SignalStack {
    stack: Stack,
}

impl SignalStack {
    pub fn new() -> io::Result<Self> {
        SignalStack {
            stack: Stack::new(libc::MINSIGSTKSZ.max(1 << 15))?,
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
        action.sa_flags = SA_SIGINFO | SA_ONSTACK;
        action.sa_sigaction = sigsegv_handler as usize;

        #[cfg(target_os = "linux")]
        let signal = libc::SIGSEGV;
        #[cfg(not(target_os = "linux"))]
        let signal = libc::SIGBUS;

        syscall!(sigaction, signal, &action, null_mut())?;
        Ok(())
    }
}

fn sigsegv_handler(_signum: i32, info: &libc::siginfo_t, _data: *mut ()) {
    let thread = pneuma::thread::current();
    let name = thread.name().unwrap_or("<unknown>");
    let stack = &thread.0.stack;

    if stack.is_stackoverflow(unsafe { info.si_addr() }) {
        println!("green thread '{name}' has overflowed its stack",);
    } else {
        println!("Segmentation fault");
    }
    std::process::abort();
}

impl Drop for SignalStack {
    fn drop(&mut self) {
        let mut ss = self.stack.stack_t();
        ss.ss_flags = SS_DISABLE;
        syscall!(sigaltstack, &ss, null_mut()).unwrap();
    }
}
