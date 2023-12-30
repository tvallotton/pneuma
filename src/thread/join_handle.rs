use std::{io, marker::PhantomData};

use super::{builder::Builder, RcContext};

pub struct JoinHandle<T>(pub(crate) RcContext, PhantomData<T>);

impl<T> JoinHandle<T> {
    pub(crate) fn new<F>(f: F, builder: Builder) -> io::Result<Self>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let cx = RcContext::new(f, builder)?;
        Ok(JoinHandle(cx, PhantomData))
    }
}
