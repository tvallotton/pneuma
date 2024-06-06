use mio::Interest;
use std::{
    fmt::Debug,
    io::{self, Error, IoSlice},
    net::{Shutdown, SocketAddr},
    os::fd::AsRawFd,
    time::{Duration, Instant},
};

use pneuma::net::ToSocketAddrs;
use pneuma::reactor::{nonblocking::nonblocking, Registered};

use super::to_socket_addr::try_each;

pub struct TcpStream {
    registered: Registered<mio::net::TcpStream>,
}

impl TcpStream {
    /// Opens a TCP connection to a remote host.
    ///
    /// `addr` is an address of the remote host. Anything which implements
    /// [`ToSocketAddrs`] trait can be supplied for the address; see this trait
    /// documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until a connection is successful. If none of
    /// the addresses result in a successful connection, the error returned from
    /// the last connection attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Open a TCP connection to `127.0.0.1:8080`:
    ///
    /// ```no_run
    /// use pneuma::net::TcpStream;
    ///
    /// if let Ok(stream) = TcpStream::connect("127.0.0.1:8080") {
    ///     println!("Connected to the server!");
    /// } else {
    ///     println!("Couldn't connect to server...");
    /// }
    /// ```
    ///
    /// Open a TCP connection to `127.0.0.1:8080`. If the connection fails, open
    /// a TCP connection to `127.0.0.1:8081`:
    ///
    /// ```no_run
    /// use pneuma::net::{SocketAddr, TcpStream};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 8080)),
    ///     SocketAddr::from(([127, 0, 0, 1], 8081)),
    /// ];
    /// if let Ok(stream) = TcpStream::connect(&addrs[..]) {
    ///     println!("Connected to the server!");
    /// } else {
    ///     println!("Couldn't connect to server...");
    /// }
    /// ```
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        try_each(addr, |addr| Self::_connect_timeout(addr, None))
    }

    /// Opens a TCP connection to a remote host with a timeout.
    ///
    /// Unlike `connect`, `connect_timeout` takes a single [`SocketAddr`] since
    /// timeout must be applied to individual addresses.
    ///
    /// It is an error to pass a zero `Duration` to this function.
    ///
    /// Unlike other methods on `TcpStream`, this does not correspond to a
    /// single system call. It instead calls `connect` in nonblocking mode and
    /// then uses an OS-specific mechanism to await the completion of the
    /// connection request.
    pub fn connect_timeout(addr: SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
        try_each(addr, |addr| Self::_connect_timeout(addr, Some(timeout)))
    }

    #[inline]
    fn _connect_timeout(addr: SocketAddr, timeout: Option<Duration>) -> io::Result<TcpStream> {
        // source: https://github.com/Thomasdezeeuw/heph/blob/0c4f1ab3eaf08bea1d65776528bfd6114c9f8374/src/net/tcp/stream.rs#L560-L622
        // This relates directly Mio and `kqueue(2)` and `epoll(2)`. To do a
        // non-blocking TCP connect properly we need to a couple of things.
        //
        // 1. Setup a socket and call `connect(2)`. Mio does this for us.
        //    However it doesn't mean the socket is connected, as we can't
        //    determine that without blocking.
        // 2. To determine if a socket is connected we need to wait for a
        //    `kqueue(2)`/`epoll(2)` event (we get scheduled once we do). But
        //    that doesn't tell us whether or not the socket is connected. To
        //    determine if the socket is connected we need to use `getpeername`
        //    (`TcpStream::peer_addr`). But before checking if we're connected
        //    we need to check for a connection error, by checking `SO_ERROR`
        //    (`TcpStream::take_error`) to not lose that information.
        //    However if we get an event (and thus get scheduled) and
        //    `getpeername` fails with `ENOTCONN` it doesn't actually mean the
        //    socket will never connect properly. So we loop (by returned
        //    `Poll::Pending`) until either `SO_ERROR` is set or the socket is
        //    connected.
        //
        // Sources:
        // * https://cr.yp.to/docs/connect.html
        // * https://stackoverflow.com/questions/17769964/linux-sockets-non-blocking-connect

        let stream = mio::net::TcpStream::connect(addr)?;

        let registered = Registered::register(stream, Interest::READABLE | Interest::WRITABLE)?;

        let start = timeout.map(|_| Instant::now());

        loop {
            // If we hit an error while connecting return that error.
            if let Ok(Some(err)) | Err(err) = registered.source.take_error() {
                return Err(err);
            }

            // If we can get a peer address it means the stream is
            // connected.
            let Err(err) = registered.source.peer_addr() else {
                return Ok(TcpStream { registered });
            };

            // `NotConnected` (`ENOTCONN`) means the socket not yet
            // connected, but still working on it. `ECONNREFUSED` will
            // be reported if it fails.
            if err.kind() != io::ErrorKind::NotConnected
                && err.raw_os_error() != Some(libc::EINPROGRESS)
            {
                return Err(err);
            }

            // Check if we have exceeded the timeout.
            if start.is_some_and(|time| time.elapsed() > timeout.unwrap()) {
                return Err(Error::from_raw_os_error(libc::ETIMEDOUT));
            }

            // Socket is not (yet) connected but haven't hit an
            // error either. So we yield and wait for
            // another event.
            pneuma::uthread::park()?;
        }
    }
    /// Returns the socket address of the remote peer of this TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use pneuma::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
    /// let stream = TcpStream::connect("127.0.0.1:8080")?;
    ///
    /// assert_eq!(stream.peer_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)));
    /// # Ok(())}
    /// ```
    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {
        self.stream().peer_addr()
    }

    /// Returns the socket address of the local half of this TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use pneuma::net::{IpAddr, Ipv4Addr, TcpStream};
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// assert_eq!(stream.local_addr().unwrap().ip(),
    ///            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    /// # Ok(())}
    /// ```
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream().local_addr()
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_nodelay`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_nodelay(true).expect("set_nodelay call failed");
    /// assert_eq!(stream.nodelay().unwrap_or(false), true);
    /// ```
    pub fn nodelay(&self) -> io::Result<bool> {
        self.stream().nodelay()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that
    /// segments are always sent as soon as possible, even if there is only a
    /// small amount of data. When not set, data is buffered until there is a
    /// sufficient amount to send out, thereby avoiding the frequent sending of
    /// small packets.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::net::TcpStream;
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_nodelay(true).expect("set_nodelay call failed");
    /// ```
    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.stream().set_nodelay(nodelay)
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value (see the
    /// documentation of [`Shutdown`]).
    ///
    /// # Platform-specific behavior
    ///
    /// Calling this function multiple times may result in different behavior,
    /// depending on the operating system. On Linux, the second call will
    /// return `Ok(())`, but on macOS, it will return `ErrorKind::NotConnected`.
    /// This may change in the future.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::net::{Shutdown, TcpStream};
    ///# fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("127.0.0.1:8080")?;
    ///                        
    /// stream.shutdown(Shutdown::Both).expect("shutdown call failed");
    /// # Ok(())}
    /// ```
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.stream().shutdown(how)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        nonblocking(|| self.stream().peek(buf), None)
    }

    fn stream(&self) -> &mio::net::TcpStream {
        &self.registered.source
    }
}

impl io::Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.registered.add(Interest::WRITABLE)?;
        nonblocking(|| self.stream().write(buf), None)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.registered.add(Interest::WRITABLE)?;
        nonblocking(|| self.stream().write_vectored(bufs), None)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.registered.add(Interest::READABLE)?;
        nonblocking(|| self.stream().read(buf), None)
    }
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.registered.add(Interest::READABLE)?;
        nonblocking(|| self.stream().read_vectored(bufs), None)
    }
}

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.stream().as_raw_fd()
    }
}

impl Debug for TcpStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.stream().fmt(f)
    }
}
