use std::future::Future;

mod waker;

fn wait<F: Future>(f: F) -> F::Output {
    // let f = std::pin::pin!(f);
    todo!()
}
