use std::{
    io::{self, Error},
    mem::transmute,
};

use io_uring::squeue;

use crate::thread::Thread;

pub fn submit(sqe: squeue::Entry) -> io::Result<i32> {
    let rt = pneuma::runtime::current();
    let thread = rt.executor.current();
    let sqe = sqe.user_data(unsafe { transmute(thread.clone()) });
    rt.reactor.push(sqe)?;
    let start = std::time::Instant::now();
    loop {
        if thread.is_cancelled() {
            return Err(Error::from_raw_os_error(libc::ECANCELED));
        }

        let Some(io_result) = thread.io_result().take() else {
            dbg!(std::panic::Location::caller());
            dbg!(pneuma::thread::park());
            continue;
        };
        dbg!(start.elapsed());

        if io_result.is_negative() {
            return Err(Error::from_raw_os_error(-io_result));
        }

        return Ok(io_result);
    }
}
