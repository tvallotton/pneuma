use std::{
    fmt::Debug,
    io,
    net::{Ipv4Addr, Ipv6Addr},
    os::fd::{AsRawFd, FromRawFd},
    time::Duration,
};

use mio::Interest;
use pneuma::reactor::{nonblocking::nonblocking, Registered};
use std::net::SocketAddr;

use super::to_socket_addr::{try_each, ToSocketAddrs};

pub struct UdpSocket {
    registered: Registered<mio::net::UdpSocket>,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        try_each(addr, Self::_bind)
    }

    pub fn _bind(addr: SocketAddr) -> io::Result<UdpSocket> {
        let socket = mio::net::UdpSocket::bind(addr)?;

        let registered = Registered::register(socket, Interest::READABLE | Interest::WRITABLE)?;

        Ok(UdpSocket {
            registered,
            read_timeout: None,
            write_timeout: None,
        })
    }
    fn socket(&self) -> &mio::net::UdpSocket {
        &self.registered.source
    }

    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        try_each(addr, |addr| {
            nonblocking(|| self.socket().connect(addr), None)
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket().local_addr()
    }

    pub fn send_to(&self, buf: &[u8], target: &SocketAddr) -> io::Result<usize> {
        nonblocking(|| self.socket().send_to(buf, *target), None)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        nonblocking(|| self.socket().send(buf), None)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        nonblocking(|| self.socket().recv(buf), self.read_timeout)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        nonblocking(|| self.socket().recv_from(buf), self.read_timeout)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.socket().peer_addr()
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        nonblocking(|| self.socket().peek_from(buf), None)
    }

    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use pneuma::net::UdpSocket;
    ///
    /// let broadcast_socket = UdpSocket::bind("127.0.0.1:0")?;
    /// if broadcast_socket.broadcast()? == false {
    ///     broadcast_socket.set_broadcast(true)?;
    /// }
    ///
    /// assert_eq!(broadcast_socket.broadcast()?, true);
    /// #
    /// #    Ok(()) }
    /// ```
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.socket().set_broadcast(on)
    }

    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see
    /// [`set_broadcast`][link].
    ///
    /// [link]: #method.set_broadcast
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use pneuma::net::UdpSocket;
    ///
    /// let broadcast_socket = UdpSocket::bind("127.0.0.1:0")?;
    /// assert_eq!(broadcast_socket.broadcast()?, false);
    /// #
    /// #    Ok(()) }
    /// ```
    pub fn broadcast(&self) -> io::Result<bool> {
        self.socket().broadcast()
    }

    /// Sets the value of the `IP_MULTICAST_LOOP` option for this socket.
    ///
    /// If enabled, multicast packets will be looped back to the local socket.
    /// Note that this may not have any affect on IPv6 sockets.
    pub fn set_multicast_loop_v4(&self, on: bool) -> io::Result<()> {
        self.socket().set_multicast_loop_v4(on)
    }

    /// Sets the value of the `IP_MULTICAST_TTL` option for this socket.
    ///
    /// Indicates the time-to-live value of outgoing multicast packets for
    /// this socket. The default value is 1 which means that multicast packets
    /// don't leave the local network unless explicitly requested.
    ///
    /// Note that this may not have any affect on IPv6 sockets.
    pub fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()> {
        self.socket().set_multicast_ttl_v4(ttl)
    }

    /// Gets the value of the `IP_MULTICAST_LOOP` option for this socket.
    ///
    /// For more information about this option, see
    /// [`set_multicast_loop_v4`][link].
    ///
    /// [link]: #method.set_multicast_loop_v4
    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.socket().multicast_loop_v4()
    }

    /// Gets the value of the `IP_MULTICAST_TTL` option for this socket.
    ///
    /// For more information about this option, see
    /// [`set_multicast_ttl_v4`][link].
    ///
    /// [link]: #method.set_multicast_ttl_v4
    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        self.socket().multicast_ttl_v4()
    }

    /// Sets the value of the `IPV6_MULTICAST_LOOP` option for this socket.
    ///
    /// Controls whether this socket sees the multicast packets it sends itself.
    /// Note that this may not have any affect on IPv4 sockets.
    pub fn set_multicast_loop_v6(&self, on: bool) -> io::Result<()> {
        self.socket().set_multicast_loop_v6(on)
    }

    /// Gets the value of the `IPV6_MULTICAST_LOOP` option for this socket.
    ///
    /// For more information about this option, see
    /// [`set_multicast_loop_v6`][link].
    ///
    /// [link]: #method.set_multicast_loop_v6
    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.socket().multicast_loop_v6()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use pneuma::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:0")?;
    /// if socket.ttl()? < 255 {
    ///     socket.set_ttl(255)?;
    /// }
    ///
    /// assert_eq!(socket.ttl()?, 255);
    /// #
    /// #    Ok(())
    /// # }
    /// ```
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.socket().set_ttl(ttl)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`set_ttl`][link].
    ///
    /// [link]: #method.set_ttl
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use pneuma::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:0")?;
    /// socket.set_ttl(255)?;
    ///
    /// assert_eq!(socket.ttl()?, 255);
    /// #
    /// #    Ok(())
    /// # }
    /// ```
    pub fn ttl(&self) -> io::Result<u32> {
        self.socket().ttl()
    }

    /// Executes an operation of the `IP_ADD_MEMBERSHIP` type.
    ///
    /// This function specifies a new multicast group for this socket to join.
    /// The address must be a valid multicast address, and `interface` is the
    /// address of the local interface with which the system should join the
    /// multicast group. If it's equal to `INADDR_ANY` then an appropriate
    /// interface is chosen by the system.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.socket().join_multicast_v4(multiaddr, interface)
    }

    /// Executes an operation of the `IPV6_ADD_MEMBERSHIP` type.
    ///
    /// This function specifies a new multicast group for this socket to join.
    /// The address must be a valid multicast address, and `interface` is the
    /// index of the interface to join/leave (or 0 to indicate any interface).
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.socket().join_multicast_v6(multiaddr, interface)
    }

    /// Executes an operation of the `IP_DROP_MEMBERSHIP` type.
    ///
    /// For more information about this option, see
    /// [`join_multicast_v4`][link].
    ///
    /// [link]: #method.join_multicast_v4
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> io::Result<()> {
        self.socket().leave_multicast_v4(multiaddr, interface)
    }

    /// Executes an operation of the `IPV6_DROP_MEMBERSHIP` type.
    ///
    /// For more information about this option, see
    /// [`join_multicast_v6`][link].
    ///
    /// [link]: #method.join_multicast_v6
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
        self.socket().leave_multicast_v6(multiaddr, interface)
    }

    /// Get the value of the `IPV6_V6ONLY` option on this socket.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn only_v6(&self) -> io::Result<bool> {
        self.socket().only_v6()
    }

    /// Get the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.socket().take_error()
    }

    /// Sets the read timeout to the timeout specified.
    ///
    /// If the value specified is [`None`], then [`read`] calls will wait
    /// indefinitely. An [`Err`] is returned if the zero [`Duration`] is
    /// passed to this method.
    ///
    /// # Platform-specific behavior
    ///
    /// Platforms may return a different error code whenever a read times out as
    /// a result of setting this option. For example Unix typically returns an
    /// error of the kind [`WouldBlock`], but Windows may return [`TimedOut`].
    ///
    /// [`read`]: io::Read::read
    /// [`WouldBlock`]: io::ErrorKind::WouldBlock
    /// [`TimedOut`]: io::ErrorKind::TimedOut
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::net::UdpSocket;
    ///
    /// let mut socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_read_timeout(None).expect("set_read_timeout call failed");
    /// ```
    ///
    /// An [`Err`] is returned if the zero [`Duration`] is passed to this
    /// method:
    ///
    /// ```no_run
    /// use std::io;
    /// use pneuma::net::UdpSocket;
    /// use std::time::Duration;
    ///
    /// let mut socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    /// let result = socket.set_read_timeout(Some(Duration::new(0, 0)));
    /// let err = result.unwrap_err();
    /// assert_eq!(err.kind(), io::ErrorKind::InvalidInput)
    /// ```

    pub fn set_read_timeout(&mut self, dur: Option<Duration>) -> io::Result<()> {
        self.as_std(|s| s.set_read_timeout(dur))?;
        self.read_timeout = dur;
        Ok(())
    }

    pub fn set_write_timeout(&mut self, dur: Option<Duration>) -> io::Result<()> {
        self.as_std(|s| s.set_write_timeout(dur))?;
        self.write_timeout = dur;
        Ok(())
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(self.read_timeout)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(self.write_timeout)
    }

    pub fn peek(&self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.socket().peek(buf)
    }

    pub(crate) fn as_std<T>(&self, mut f: impl FnMut(&mut std::net::UdpSocket) -> T) -> T {
        let fd = self.socket().as_raw_fd();
        let mut socket = unsafe { std::net::UdpSocket::from_raw_fd(fd) };
        let out = f(&mut socket);
        std::mem::forget(socket);
        out
    }
}

impl Debug for UdpSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_std(|s| Debug::fmt(s, f))
    }
}
