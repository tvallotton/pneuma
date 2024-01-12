use std::{
    cell::Cell,
    io,
    io::Error,
    ptr::null_mut,
    time::{Duration, Instant},
};

use libc::kevent;

use super::event::submit;

const ZEROED: kevent = kevent {
    ident: 0,
    filter: 0,
    flags: 0,
    fflags: 0,
    data: 0,
    udata: null_mut(),
};

#[inline]
pub fn sleep(dur: Duration) -> io::Result<()> {
    let mut event = ZEROED;
    event.ident += event_id();
    event.flags = libc::EV_ADD | libc::EV_ENABLE;
    event.filter = libc::EVFILT_TIMER;
    event.data = dur.as_millis() as _;
    let time = Instant::now();
    submit(event, || {
        if time.elapsed() < dur {
            Err(Error::from_raw_os_error(libc::EAGAIN))
        } else {
            Ok(())
        }
    })
}

fn event_id() -> usize {
    thread_local! {
        static EVENT_ID: Cell<usize> = Cell::default();
    }
    EVENT_ID.with(|cell| {
        let value = cell.get();
        cell.set(value + 1);
        value
    })
}
