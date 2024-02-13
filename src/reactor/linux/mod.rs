use std::{io, mem::transmute, time::Duration};

use io_uring::IoUring;

use pneuma::thread::Thread;

pub mod event;
pub mod op;

pub struct Reactor {
    io_uring: IoUring,
}

pub type Event = io_uring::squeue::Entry;

impl Reactor {
    pub fn new() -> io::Result<Reactor> {
        let io_uring = IoUring::new(256)?;
        Ok(Reactor { io_uring })
    }

    #[inline]
    pub fn push(&mut self, ev: Event) -> io::Result<()> {
        unsafe {
            let mut queue = self.io_uring.submission();
            if queue.push(&ev).is_ok() {
                return Ok(());
            }
            drop(queue);
            self.io_uring.submit()?;
            self.io_uring.submission().push(&ev).ok();
        }
        Ok(())
    }

    #[inline]
    pub fn submit_and_yield(&mut self) -> io::Result<()> {
        self.io_uring.submit_and_wait(0)?;
        Ok(())
    }

    #[inline]
    pub fn submit_and_wait(&mut self) -> io::Result<()> {
        self.submit(Duration::from_secs(10))
    }

    pub fn submit(&mut self, dur: Duration) -> io::Result<()> {
        let timespec = io_uring::types::Timespec::new()
            .sec(dur.as_secs())
            .nsec(dur.subsec_nanos());
        let args = io_uring::types::SubmitArgs::default().timespec(&timespec);
        self.io_uring.submitter().submit_with_args(1, &args)?;
        self.wake_tasks();

        Ok(())
    }

    pub fn wake_tasks(&mut self) {
        for cqe in self.io_uring.completion() {
            let option: Option<Thread> = unsafe { transmute(cqe.user_data()) };
            let Some(thread) = option else {
                println!("YEEEES");
                continue;
            };

            thread.unpark();
            thread.0.io_result.set(Some(cqe.result()));
        }
    }
}
