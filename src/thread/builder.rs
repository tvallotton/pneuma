use std::io;

use pneuma::thread::JoinHandle;

use crate::runtime::{current, Runtime};

pub struct Builder {
    pub(crate) name: Option<String>,
    pub(crate) stack_size: usize,
    pub(crate) rt: Option<Runtime>,
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl Builder {
    /// Generates the base configuration for spawning a green thread, from which
    /// configuration methods can be chained.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::thread;
    ///
    /// let builder = thread::Builder::new()
    ///                               .name("foo".into())
    ///                               .stack_size(32 * 1024);
    ///
    /// let handler = builder.spawn(|| {
    ///     // thread code
    /// }).unwrap();
    ///
    /// handler.join();
    /// ```
    pub fn new() -> Builder {
        Builder {
            name: None,
            stack_size: 1 << 16,
            rt: None,
        }
    }

    /// Names the thread-to-be. Currently the name is used for identification
    /// only in panic messages.
    ///
    /// The name must not contain null bytes (`\0`).
    ///
    /// For more information about named threads, see
    /// [this module-level documentation][naming-threads].
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::thread;
    ///
    /// let builder = thread::Builder::new()
    ///     .name("foo".into());
    ///
    /// let handler = builder.spawn(|| {
    ///     assert_eq!(thread::current().name(), Some("foo"))
    /// }).unwrap();
    ///
    /// handler.join()
    /// ```
    pub fn name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    /// Sets the size of the stack (in bytes) for the new thread.
    ///
    /// The actual stack size may be silently raised to the platforms
    /// minimum stack size.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::thread;
    ///
    /// let builder = thread::Builder::new().stack_size(32 * 1024);
    /// ```
    pub fn stack_size(self, stack_size: usize) -> Self {
        Self { stack_size, ..self }
    }

    pub fn spawn<T, F>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        current().spawn(f, self)
    }

    pub(crate) fn for_os_thread() -> Self {
        Builder {
            name: std::thread::current().name().map(Into::into),
            stack_size: 0,
            rt: None,
        }
    }

    pub(crate) fn runtime(self, rt: Runtime) -> Self {
        Builder {
            rt: Some(rt),
            ..self
        }
    }
}
