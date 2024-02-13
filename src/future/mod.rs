use std::{
    future::{Future, IntoFuture},
    task::{Context, Poll, Waker},
};

use pneuma::thread::park;

use crate::thread::JoinHandle;

pub(crate) mod thin_waker;
mod waker;

/// awaits a future asynchronously
///
/// # Example
/// ```ignore
///
/// let value = wait(async {
///     10
/// })
/// assert!(value, 10);
/// ```
pub fn wait<F: Future>(f: F) -> F::Output {
    let thread = pneuma::thread::current();
    let waker: Waker = thread.into();
    let mut cx = Context::from_waker(&waker);

    let mut f = std::pin::pin!(f);
    loop {
        if let Poll::Ready(out) = f.as_mut().poll(&mut cx) {
            return out;
        };
        park()
    }
}
