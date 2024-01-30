use std::future::Future;

pub mod thin_waker;
mod waker;

fn wait<F: Future>(_f: F) -> F::Output {
    // let f = std::pin::pin!(f);
    todo!()
}
