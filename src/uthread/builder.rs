use std::io;

use pneuma::uthread::JoinHandle;

use crate::sys::stack::Stack;

/// Thread factory, which can be used in order to configure the properties of
/// a new uthread.
///
/// Methods can be chained on it in order to configure it.
///
/// The two configurations available are:
///
/// - [`name`]: specifies an associated name for the uthread.
/// - [`stack_size`]: specifies the desired stack size for the uthread.
///
/// The [`spawn`] method will take ownership of the builder and create an
/// [`io::Result`] to the uthread handle with the given configuration.
///
/// The [`uthread::spawn`] free function uses a `Builder` with default
/// configuration and [`unwrap`]s its return value.
///
/// You may want to use [`spawn`] instead of [`uthread::spawn`], when you want
/// to recover from a failure to launch a uthread, indeed the free function will
/// panic where the `Builder` method will return a [`io::Result`].
///
/// # Examples
///
/// ```
/// use pneuma::uthread;
///
/// let builder = uthread::Builder::new();
///
/// let handler = builder.spawn(|| {
///     // thread code
/// }).unwrap();
///
/// handler.join().unwrap();
/// ```
///
/// [`stack_size`]: Builder::stack_size
/// [`name`]: Builder::name
/// [`spawn`]: Builder::spawn
/// [`uthread::spawn`]: spawn
/// [`unwrap`]: std::result::Result::unwrap
pub struct Builder {
    pub(crate) name: Option<String>,
    pub(crate) stack_size: usize,
}

impl Builder {
    /// Generates the base configuration for spawning a green uthread, from which
    /// configuration methods can be chained.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::uthread;
    ///
    /// let builder = uthread::Builder::new()
    ///     .name("foo".into())
    ///     .stack_size(32 * 1024);
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
            stack_size: 32 * 1024,
        }
    }

    /// Names the uthread-to-be. Currently the name is used for identification
    /// only in panic messages.
    ///
    /// For more information about named threads, see
    /// [this module-level documentation][naming-threads].
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::uthread;
    ///
    /// let builder = uthread::Builder::new()
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

    /// Spawns a new uthread by taking ownership of the `Builder`, and returns an
    /// [`io::Result`] to its [`JoinHandle`].
    ///
    /// The spawned uthread may outlive the caller (unless the caller thread
    /// is the main thread; the whole process is terminated when the main
    /// thread finishes). The join handle can be used to block on
    /// termination of the spawned thread, including recovering its panics.
    ///
    /// For a more complete documentation see [`pneuma::uthread::spawn`][`spawn`].
    ///
    /// # Errors
    ///
    /// Unlike the [`spawn`] free function, this method yields an
    /// [`io::Result`] to capture any failure to create the thread at
    /// the OS level.
    ///
    /// [`io::Result`]: crate::io::Result
    ///
    /// # Panics
    ///
    /// Panics if a thread name was set and it contained null bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::uthread;
    ///
    /// let builder = uthread::Builder::new();
    ///
    /// let handler = builder.spawn(|| {
    ///     // thread code
    /// }).unwrap();
    ///
    /// handler.join();
    /// ```
    pub fn spawn<T, F>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static + Send,
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

    pub(crate) fn stack(&mut self) -> io::Result<Stack> {
        let rt = pneuma::runtime::current();
        if let Some(stack) = rt.executor.stack(self.stack_size) {
            return Ok(stack);
        }
        Stack::new(self.stack_size)
    }
}
