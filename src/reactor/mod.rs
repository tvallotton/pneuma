use std::{cell::UnsafeCell, io};

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub(crate) use bsd as imp;
pub use imp::op;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) use linux as imp;

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub(crate) mod bsd;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

pub struct Reactor(UnsafeCell<imp::Reactor>);

impl Reactor {
    pub fn new() -> Reactor {
        Reactor(UnsafeCell::new(imp::Reactor::new()))
    }

    pub fn push(&self, ev: imp::Event) -> io::Result<()> {
        unsafe { &mut *self.0.get() }.push(ev)
    }

    pub fn submit_and_yield(&self) -> io::Result<()> {
        unsafe { &mut *self.0.get() }.submit_and_yield()
    }

    pub fn submit_and_wait(&self) -> io::Result<()> {
        unsafe { &mut *self.0.get() }.submit_and_wait()
    }
}
