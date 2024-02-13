#![allow(unreachable_patterns)]

use super::Event;
use libc::{EAGAIN, EINPROGRESS, EWOULDBLOCK};
use std::{
    io::{self, Error},
    mem::transmute,
};

#[inline]
pub fn submit<F, T>(event: Event, mut f: F) -> io::Result<T>
where
    F: FnMut() -> io::Result<T>,
{
    loop {
        let result = f();

        let Err(err) = result else {
            return result;
        };

        let Some(EAGAIN | EWOULDBLOCK | EINPROGRESS) = err.raw_os_error() else {
            return Err(err);
        };

        wait(event)?;
    }
}

#[inline]
pub fn wait(mut ev: Event) -> io::Result<()> {
    let runtime = pneuma::runtime::current();
    let thread = runtime.executor.current();
    if thread.is_cancelled() {
        return Err(Error::from_raw_os_error(libc::ECANCELED));
    }
    ev.udata = unsafe { transmute(thread) };
    runtime.reactor.push(ev)?;
    runtime.park();

    Ok(())
}
