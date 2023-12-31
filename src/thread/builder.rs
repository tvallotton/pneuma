use std::io;



use pneuma::thread::JoinHandle;

pub struct Builder {
    pub(crate) name: Option<String>,
    pub(crate) stack_size: usize,
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
    /// ```ignore
    /// use pneuma::thread;
    ///
    /// let builder = thread::Builder::new()
    ///                               .name("foo".into())
    ///                               .stack_size(2 * 1024);
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

            stack_size: 1 << 14,
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
    /// ```ignore
    /// use pneuma::thread;
    ///
    /// let builder = thread::Builder::new()
    ///     .name("foo".into());
    ///
    /// let handler = builder.spawn(|| {
    ///     assert_eq!(thread::current().name(), Some("foo"))
    /// }).unwrap();
    ///
    /// handler.join().unwrap();
    /// ```
    pub fn name(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    /// Sets the size of the stack (in bytes) for the new thread.
    ///
    /// The actual stack size is silently floored to the platforms
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
        JoinHandle::new(f, self)
    }

    pub(crate) fn for_os_thread() -> Self {
        Builder {
            name: std::thread::current().name().map(Into::into),

            stack_size: 0,
        }
    }
}
