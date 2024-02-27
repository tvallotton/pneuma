use std::{future::poll_fn, task::Poll};

use pneuma::future::wait;

async fn yield_now() {
    let mut yielded = false;
    poll_fn(|cx| {
        if yielded {
            return Poll::Ready(());
        }
        yielded = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await
}

#[test]
pub fn test_wait_trivial() {
    assert_eq!(wait(async { 10 }), 10);
}

#[test]
pub fn test_wait() {
    let output = wait(async {
        yield_now().await;
        103
    });
    assert_eq!(output, 103);
}
