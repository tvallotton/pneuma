use pneuma::thread::{current, park};
use std::{
    collections::VecDeque,
    fmt::{self, Debug, Display, Formatter, Write},
    ops::{Deref, DerefMut},
    sync::PoisonError,
    task::Waker,
};

/// A nonblocking FIFO `Mutex`-like type.
///
/// This type acts similarly to [`std::sync::Mutex`], with two major
/// differences: [`lock`] is a yielding method, so does not block, and the lock
/// guard is designed to be held across yield points.
///
/// This struct is the green thread analog to [tokio's async mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html).
///
/// # Which kind of mutex should you use?
///
///
/// If the [`MutexGuard`] needs to be held across a yield point, then [`pneuma::sync::Mutex`]
/// should be used. Otherwise, the [`std::sync::Mutex`] should be preferred.
///
///
/// The feature that the nonblocking mutex offers over the blocking mutex is the
/// ability to keep it locked across yield points. This prevents deadlocks when
/// another green thread attempts to lock the mutex from the same OS thread.
///
/// The nonblocking mutex is more expensive than the blocking one.
/// The primary use case for the  nonblocking mutex is to provide shared mutable
/// access to IO resources such as a database connection. If the value behind the
/// mutex is just data, it's usually appropriate to use a blocking mutex such as
/// the one in the standard library or [`parking_lot`].
///  
///
/// Note that, although the compiler will not prevent the std `Mutex` from holding
/// its guard across yield points, this virtually never leads to correct concurrent
/// code in practice as it can easily lead to deadlocks.
///
/// A common pattern is to wrap the `Arc<Mutex<...>>` in a struct that provides
/// non-async methods for performing operations on the data within, and only
/// lock the mutex inside these methods.
///
/// Additionally, when you _do_ want shared access to an IO resource, it is
/// often better to spawn a task to manage the IO resource, and to use message
/// passing to communicate with that task.
///
/// [std]: std::sync::Mutex
/// [`parking_lot`]: https://docs.rs/parking_lot
/// [mini-redis]: https://github.com/tokio-rs/mini-redis/blob/master/src/db.rs
///
/// # Examples:
///
/// ```rust,no_run
/// use pneuma::{sync::Mutex, thread};
/// use std::sync::Arc;
///
///
/// let data1  = Arc::new(Mutex::new(0));
/// let data2  = Arc::clone(&data1);
///
/// let handle = thread::spawn(move || {
///     let mut lock = data2.lock();
///     thread::yield_now();
///     *lock += 1;
/// });
/// thread::yield_now();
/// let mut lock = data1.lock();
/// *lock += 1;
///
/// handle.join();
///
/// ```
///
///
/// ```rust,no_run
/// use pneuma::{sync::Mutex, thread};
/// use std::sync::Arc;
///
/// let count = Arc::new(Mutex::new(0));
///
/// for i in 0..5 {
///     let my_count = Arc::clone(&count);
///     thread::spawn(move || {
///         for j in 0..10 {
///             let mut lock = my_count.lock();
///             *lock += 1;
///             println!("{} {} {}", i, j, lock);
///         }
///     });
/// }
/// thread::yield_now();
/// loop {
///     if *count.lock() >= 50 {
///         break;
///     }
/// }
/// println!("Count hit 50.");
///
/// ```
/// There are a few things of note here to pay attention to in this example.
/// 1. The mutex is wrapped in an [`Arc`] to allow it to be shared across
///    threads.
/// 2. Each spawned task obtains a lock and releases it on every iteration.
/// 3. Mutation of the data protected by the Mutex is done by de-referencing
///    the obtained lock as seen on lines 13 and 20.
///
/// Note that in contrast to [`std::sync::Mutex`], this implementation does not
/// poison the mutex when a thread holding the [`MutexGuard`] panics. In such a
/// case, the mutex will be unlocked. If the panic is caught, this might leave
/// the data protected by the mutex in an inconsistent state.
///
/// [`Mutex`]: struct@Mutex
/// [`MutexGuard`]: struct@MutexGuard
/// [`Arc`]: struct@std::sync::Arc
/// [`std::sync::Mutex`]: struct@std::sync::Mutex
/// [`Send`]: trait@std::marker::Send
/// [`lock`]: method@Mutex::lock
#[derive(Default)]
pub struct Mutex<T> {
    queue: std::sync::Mutex<VecDeque<Waker>>,
    data: std::sync::Mutex<T>,
}

pub struct MutexGuard<'a, T>(std::sync::MutexGuard<'a, T>, &'a Mutex<T>);

#[derive(Debug, Clone, Copy)]
pub struct TryLockError;

impl Display for TryLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

enum Status {
    Unsubscribed,
    Subscribed,
    FirstInQueue,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Mutex<T> {
        let data = std::sync::Mutex::new(value);
        let queue = std::sync::Mutex::new(VecDeque::new());
        Mutex { data, queue }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        let mut state = Status::Unsubscribed;

        self.subscribe(&mut state);

        loop {
            if let Ok(guard) = self.try_lock() {
                return guard;
            }

            self.subscribe(&mut state);
            park()
        }
    }

    fn subscribe(&self, status: &mut Status) {
        let mut queue;
        match status {
            Status::Subscribed => return,

            Status::Unsubscribed => {
                queue = self.queue.lock().unwrap();
                if queue.is_empty() {
                    *status = Status::FirstInQueue;
                    return;
                }
            }
            Status::FirstInQueue => {
                queue = self.queue.lock().unwrap();
            }
        }
        queue.push_back(current().into());
        *status = Status::Subscribed;
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn try_lock(&self) -> Result<MutexGuard<T>, TryLockError> {
        match self.data.try_lock() {
            Ok(value) => Ok(MutexGuard(value, self)),
            Err(std::sync::TryLockError::WouldBlock) => Err(TryLockError),
            Err(std::sync::TryLockError::Poisoned(err)) => Ok(MutexGuard(err.into_inner(), self)),
        }
    }

    pub fn into_inner(self) -> T {
        self.data
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        let Some(waker) = self.1.queue.lock().unwrap().pop_front() else {
            return;
        };
        waker.wake();
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: Debug> Debug for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value: &T = self;
        write!(f, "{:?}", value)
    }
}

impl<'a, T: Display> Display for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value: &T = self;
        write!(f, "{}", value)
    }
}

impl<T: Debug> Debug for Mutex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Ok(data) = self.try_lock() {
            f.debug_struct("Mutex").field("data", &data).finish()
        } else {
            f.debug_struct("Mutex").field("data", &"<locked>").finish()
        }
    }
}
