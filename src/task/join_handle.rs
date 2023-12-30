use std::{io, marker::PhantomData};

use super::RcContext;

pub struct JoinHandle<T>(pub(crate) RcContext, PhantomData<T>);

impl<T> JoinHandle<T> {
    pub(crate) fn new<F>(size: usize, f: F) -> io::Result<Self>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let cx = RcContext::new(size, f)?;
        Ok(JoinHandle(cx, PhantomData))
    }
}
