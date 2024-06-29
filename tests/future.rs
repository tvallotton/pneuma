use pneuma::future::wait;
use std::future::{poll_fn, Future};
use std::task::Poll;

// a future that returns pending once.
fn yield_now() -> impl Future<Output = ()> + Unpin {
    let mut yielded = false;
    poll_fn(move |cx| {
        if !yielded {
            yielded = true;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        Poll::Ready(())
    })
}

#[test]
pub fn smoke_test() {
    let out = wait(async {
        yield_now().await;
        10
    });
    assert_eq!(out, 10);
}
