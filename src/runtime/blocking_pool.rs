use std::sync::atomic::Ordering::Relaxed;
use std::{
    mem::{transmute, MaybeUninit},
    sync::atomic::AtomicU16,
};
const MAX_THREADS: u16 = 512;

pub struct BlockingPool {
    live_workers: AtomicU16,
    rendezvous: Channel,
    unbounded: Channel,
}

pub struct Channel {
    sender: crossbeam_channel::Sender<Payload<'static>>,
    receiver: crossbeam_channel::Receiver<Payload<'static>>,
}

pub struct Payload<'a> {
    function: *mut (dyn FnMut() + 'a),
}

impl BlockingPool {
    pub fn new() -> BlockingPool {
        let rendezvous = Channel::rendezvous();
        let unbounded = Channel::unbounded();

        BlockingPool {
            live_workers: 0.into(),
            rendezvous,
            unbounded,
        }
    }

    pub fn blocking<'a, T: 'a>(&self, f: impl FnMut() -> T + 'a) -> T {
        let mut output = None;

        let mut f = unsafe { type_errase(f, &mut output) };

        self.blocking_mut(&mut f);

        loop {
            if let Some(output) = output {
                return output;
            }
            pneuma::uthread::park();
        }
    }

    fn blocking_mut<'a>(&self, function: *mut (dyn FnMut() + 'a)) {
        let payload = Payload { function };
        let mut payload: Payload<'static> = unsafe { transmute(payload) };

        let Err(err) = self.sender.try_send(payload) else {
            return;
        };
        self.spawn_worker(err.into_inner());
    }

    pub fn spawn_worker(&self, payload: Payload) {
        let current = self.live_workers.fetch_add(1, Relaxed);

        if current > MAX_THREADS {
            self.live_workers.fetch_sub(1, Relaxed);
            self.unbounded.sender.send(payload);
            return;
        }
    }
}

/// Safety: the output returned closure should not outlive output. Also, output should be a
/// valid pointer
unsafe fn type_errase<'a, F, T>(f: F, output: *mut Option<T>) -> impl FnMut() + 'a
where
    F: FnOnce() -> T + 'a,
    T: 'a,
{
    let mut f = Some(f);
    move || {
        f.take().map(|f| {
            *output = Some(f());
        });
    }
}

impl Channel {
    fn unbounded() -> Channel {
        let (sender, receiver) = crossbeam_channel::unbounded();
        Channel { sender, receiver }
    }

    fn rendezvous() -> Channel {
        let (sender, receiver) = crossbeam_channel::bounded(0);
        Channel { sender, receiver }
    }
}

unsafe impl<'a> Send for Payload<'a> {}
