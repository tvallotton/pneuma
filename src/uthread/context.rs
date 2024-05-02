use std::{
    io,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::{builder::Builder, repr_context::ReprContext};

pub(crate) struct Context {
    pub ptr: NonNull<ReprContext>,
}

impl Context {
    pub fn new<F>(f: F, builder: Builder) -> io::Result<Context> {
        todo!()
    }
}

impl Deref for Context {
    type Target = ReprContext;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}
