use std::{
    fmt::{self, Debug},
    io::{self},
    net::SocketAddr,
};

use mio::Interest;

use crate::reactor::{op, Registered};

/// A TCP socket server, listening for connections.
///
/// After creating a `TcpListener` by [`bind`]ing it to a socket address, it listens
/// for incoming TCP connections. These can be accepted by calling [`accept`] or by
/// iterating over the [`Incoming`] iterator returned by [`incoming`][`TcpListener::incoming`].
///
/// The socket will be closed when the value is dropped.
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [`accept`]: TcpListener::accept
/// [`bind`]: TcpListener::bind
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
///
/// # Examples
///
/// ```no_run
/// use pneuma::net::{TcpListener, TcpStream};
///
/// fn handle_client(stream: TcpStream) {
///     // ...
/// }
///
/// fn main() -> std::io::Result<()> {
///     let listener = TcpListener::bind("127.0.0.1:80")?;
///
///     // accept connections and process them serially
///     for stream in listener.incoming() {
///         handle_client(stream?);
///     }
///     Ok(())
/// }
/// ```

pub struct TcpListener {
    registered: Registered<mio::net::TcpListener>,
}

/// An iterator that infinitely [`accept`]s connections on a [`TcpListener`].
///
/// This `struct` is created by the [`TcpListener::incoming`] method.
/// See its documentation for more.
///
/// [`accept`]: TcpListener::accept
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

use super::{to_socket_addr::try_each, TcpStream, ToSocketAddrs};

impl TcpListener {
    /// Creates a new `TcpListener` which will be bound to the specified
    /// address.
    ///
    /// The returned listener is ready for accepting connections.
    ///
    /// Binding with a port number of 0 will request that the OS assigns a port
    /// to this listener. The port allocated can be queried via the
    /// [`TcpListener::local_addr`] method.
    ///
    /// The address type can be any implementor of [`ToSocketAddrs`] trait. See
    /// its documentation for concrete examples.
    ///
    /// If `addr` yields multiple addresses, `bind` will be attempted with
    /// each of the addresses until one succeeds and returns the listener. If
    /// none of the addresses succeed in creating a listener, the error returned
    /// from the last attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a TCP listener bound to `127.0.0.1:80`:
    ///
    /// ```
    /// use pneuma::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    /// ```
    ///
    /// Creates a TCP listener bound to `127.0.0.1:80`. If that fails, create a
    /// TCP listener bound to `127.0.0.1:443`:
    ///
    /// ```
    /// use pneuma::net::{SocketAddr, TcpListener};
    ///
    /// let addrs = [
    ///     SocketAddr::from(([127, 0, 0, 1], 80)),
    ///     SocketAddr::from(([127, 0, 0, 1], 443)),
    /// ];
    /// let listener = TcpListener::bind(&addrs[..]).unwrap();
    /// ```
    ///
    /// Creates a TCP listener bound to a port assigned by the operating system
    /// at `127.0.0.1`.
    ///
    /// ```
    /// use pneuma::net::TcpListener;
    ///
    /// let socket = TcpListener::bind("127.0.0.1:0").unwrap();
    /// ```
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        try_each(addr, |sockaddr| {
            let listener = mio::net::TcpListener::bind(sockaddr)?;
            let registered = Registered::register(listener, Interest::READABLE)?;
            Ok(TcpListener { registered })
        })
    }

    /// Returns the local socket address of this listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8084").unwrap();
    /// assert_eq!(listener.local_addr().unwrap(),
    ///            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8084)));
    /// ```
    pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.registered.source.local_addr()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::net::TcpListener;
    ///
    /// let stream = TcpListener::bind("127.0.0.1:8085")
    ///                        .expect("Couldn't connect to the server...");
    /// stream.set_ttl(100).expect("set_ttl call failed");
    /// ```
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.registered.source.set_ttl(ttl)
    }

    /// Gets the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::net::TcpListener;
    ///
    /// let listener = TcpListener::bind("127.0.0.1:8086").unwrap();
    /// listener.take_error().expect("No error was expected");
    /// ```
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.registered.source.take_error()
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`TcpStream::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::net::TcpListener;
    ///
    /// let stream = TcpListener::bind("127.0.0.1:8087")
    ///     .expect("Couldn't connect to the server...");
    /// stream.set_ttl(100).expect("set_ttl call failed");
    /// assert_eq!(stream.ttl().unwrap_or(0), 100);
    /// ```
    pub fn ttl(&self) -> io::Result<u32> {
        self.registered.source.ttl()
    }

    pub fn accept(&self) -> Result<(TcpStream, SocketAddr), io::Error> {
        let (stream, addr) = self._accept()?;
        let registered = Registered::register(stream, Interest::READABLE)?;
        let stream = pneuma::net::TcpStream { registered };
        Ok((stream, addr))
    }

    fn _accept(&self) -> Result<(mio::net::TcpStream, SocketAddr), io::Error> {
        op::nonblocking(|| self.registered.source.accept(), None)
    }

    /// Returns an iterator over the connections being received on this
    /// listener.
    ///
    /// The returned iterator will never return [`None`] and will also not yield
    /// the peer's [`SocketAddr`] structure. Iterating over it is equivalent to
    /// calling [`TcpListener::accept`] in a loop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::{TcpListener, TcpStream};
    ///
    /// fn handle_connection(stream: TcpStream) {
    ///    //...
    /// }
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let listener = TcpListener::bind("127.0.0.1:80")?;
    ///
    ///     for stream in listener.incoming() {
    ///         match stream {
    ///             Ok(stream) => {
    ///                 handle_connection(stream);
    ///             }
    ///             Err(e) => { /* connection failed */ }
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn incoming(&self) -> Incoming {
        Incoming { listener: self }
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.registered.source.fmt(f)
    }
}

impl Iterator for Incoming<'_> {
    type Item = io::Result<TcpStream>;
    fn next(&mut self) -> Option<io::Result<TcpStream>> {
        Some(self.listener.accept().map(|p| p.0))
    }
}

#[test]
fn incoming() {
    use std::io::{Read, Write};
    let addr = "127.0.0.1:80";

    let listener = TcpListener::bind(addr).unwrap();

    let handle = pneuma::uthread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        write!(&mut stream, "hello world").unwrap();
    });

    for result in listener.incoming() {
        let mut stream = result.unwrap();

        let mut string = String::new();

        stream.read_to_string(&mut string).unwrap();

        assert_eq!(string, "hello world");

        break;
    }
    handle.join();
}
