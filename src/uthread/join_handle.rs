use std::{
    any::Any,
    io,
    marker::PhantomData,
    panic::resume_unwind,
    sync::atomic::Ordering,
};

use super::{
    builder::Builder,
    lifecycle::{FINISHED, NEW, RUNNING, TAKEN},
    Context, UThread,
};

/// An owned permission to join on a green thread (wait on its termination).
///
/// A `JoinHandle` *detaches* the associated thread when it is dropped, which
/// means that there is no longer any handle to the thread and no way to `join`
/// on it.
///
/// Due to ownership restrictions, it is not possible to [`Clone`] this
/// handle: the ability to join a thread is a uniquely-owned permission.
///
/// This `struct` is created by the [`uthread::spawn`] function and the
/// [`uthread::Builder::spawn`] method.
///
/// # Examples
///
/// Creation from [`thread::spawn`]:
///
/// ```
/// use pneuma::uthread;
///
/// let join_handle: uthread::JoinHandle<_> = uthread::spawn(|| {
///     // some work here
/// });
/// ```
///
/// Creation from [`uthread::Builder::spawn`]:
///
/// ```
/// use pneuma::uthread;
///
/// let builder = uthread::Builder::new();
///
/// let join_handle: uthread::JoinHandle<_> = builder.spawn(|| {
///     // some work here
/// }).unwrap();
/// ```
///
/// A thread being detached and outliving the thread that spawned it:
///
/// ```no_run
/// use pneuma::uthread;
/// use pneuma::time::{Duration, sleep};
///
/// let original_thread = uthread::spawn(|| {
///     let _detached_thread = uthread::spawn(|| {
///         // Here we sleep to make sure that the first thread returns before.
///         sleep(Duration::from_millis(10));
///         // This will be called, even though the JoinHandle is dropped.
///         println!("♫ Still alive ♫");
///     });
/// });
///
/// original_thread.join();
/// println!("Original thread is joined.");
///
/// // We make sure that the new thread has time to run, before the main
/// // thread returns.
///
/// sleep(Duration::from_millis(1000));
/// ```
///
/// [`thread::Builder::spawn`]: Builder::spawn
/// [`thread::spawn`]: spawn
pub struct JoinHandle<T> {
    pub(crate) thread: UThread,
    pub(crate) _ph: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    pub(crate) fn new<F>(f: F, builder: Builder) -> io::Result<Self>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let cx = Context::new(f, builder)?;
        let thread = UThread { cx };
        thread.unpark();
        let _ph = PhantomData;
        Ok(JoinHandle { thread, _ph })
    }

    pub fn join(self) -> T {
        match self.try_join() {
            Ok(out) => out,
            Err(err) => resume_unwind(err),
        }
    }

    pub fn thread(&self) -> &UThread {
        &self.thread
    }

    /// Checks if the associated thread has finished running its main function.
    ///
    /// `is_finished` supports implementing a non-blocking join operation, by checking
    /// `is_finished`, and calling `join` if it returns `false`. This function does not block. To
    /// block while waiting on the thread to finish, use [`join`][Self::join].
    ///
    /// This might return `true` for a brief moment after the thread's main
    /// function has returned, but before the thread itself has stopped running.
    /// However, once this returns `true`, [`join`][Self::join] can be expected
    /// to return quickly, without blocking for any significant amount of time.
    pub fn is_finished(&self) -> bool {
        self.thread.cx.lifecycle.load(Ordering::Relaxed) == FINISHED
    }

    #[allow(unused_must_use)]
    pub fn try_join(self) -> Result<T, Box<dyn Any + Send + 'static>> {
        loop {
            let lifecycle = &self.thread.cx.lifecycle;
            match lifecycle.load(Ordering::Acquire) {
                NEW | RUNNING => {
                    self.thread
                        .cx
                        .join_waker
                        .lock()
                        .unwrap()
                        .insert(pneuma::uthread::current());
                    self.thread.unpark();
                    pneuma::uthread::park().unwrap();
                }
                FINISHED => unsafe {
                    lifecycle.store(TAKEN, Ordering::Release);

                    let out: *mut Result<T, Box<dyn Any + Send + 'static>> =
                        self.thread.cx.out.cast();
                    return out.read();
                },
                _ => unreachable!(),
            }
        }
    }
}
