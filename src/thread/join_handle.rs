use std::{any::Any, io, marker::PhantomData, panic::resume_unwind};



use super::{builder::Builder, context::Lifecycle, RcContext, Thread};

/// An owned permission to join on a green thread (block on its termination).
///
/// A `JoinHandle` *detaches* the associated thread when it is dropped, which
/// means that there is no longer any handle to the thread and no way to `join`
/// on it.
///
/// Due to ownership restrictions, it is not possible to [`Clone`] this
/// handle: the ability to join a thread is a uniquely-owned permission.
///
/// This `struct` is created by the [`thread::spawn`] function and the
/// [`thread::Builder::spawn`] method.
///
/// # Examples
///
/// Creation from [`thread::spawn`]:
///
/// ```ignore
/// use pneuma::thread;
///
/// let join_handle: thread::JoinHandle<_> = thread::spawn(|| {
///     // some work here
/// });
/// ```
///
/// Creation from [`thread::Builder::spawn`]:
///
/// ```ignore
/// use pneuma::thread;
///
/// let builder = thread::Builder::new();
///
/// let join_handle: thread::JoinHandle<_> = builder.spawn(|| {
///     // some work here
/// }).unwrap();
/// ```
///
/// A thread being detached and outliving the thread that spawned it:
///
/// ```no_run
/// use std::thread;
/// use std::time::Duration;
///
/// let original_thread = thread::spawn(|| {
///     let _detached_thread = thread::spawn(|| {
///         // Here we sleep to make sure that the first thread returns before.
///         thread::sleep(Duration::from_millis(10));
///         // This will be called, even though the JoinHandle is dropped.
///         println!("♫ Still alive ♫");
///     });
/// });
///
/// original_thread.join().expect("The thread being joined has panicked");
/// println!("Original thread is joined.");
///
/// // We make sure that the new thread has time to run, before the main
/// // thread returns.
///
/// thread::sleep(Duration::from_millis(1000));
/// ```
///
/// [`thread::Builder::spawn`]: Builder::spawn
/// [`thread::spawn`]: spawn
pub struct JoinHandle<T>(pub(crate) Thread, PhantomData<T>);

impl<T> JoinHandle<T> {
    pub(crate) fn new<F>(f: F, builder: Builder) -> io::Result<Self>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let cx = RcContext::new(f, builder)?;
        let thread = Thread(cx);
        Ok(JoinHandle(thread, PhantomData))
    }

    pub fn join(self) -> T {
        match self.try_join() {
            Ok(out) => out,
            Err(err) => resume_unwind(err),
        }
    }

    pub fn thread(&self) -> &Thread {
        &self.0
    }

    pub fn try_join(self) -> Result<T, Box<dyn Any + Send + 'static>> {
        loop {
            match self.0 .0.lifecycle.get() {
                Lifecycle::Taken | Lifecycle::OsThread => unreachable!(),
                Lifecycle::New | Lifecycle::Running => {
                    self.0 .0.join_waker.set(Some(pneuma::thread::current()));
                    pneuma::thread::park()
                }
                Lifecycle::Finished => unsafe {
                    self.0 .0.lifecycle.set(Lifecycle::Taken);
                    let out = self.0 .0.out as *mut Result<T, Box<dyn Any + Send + 'static>>;
                    return out.read();
                },
            }
        }
    }
}
