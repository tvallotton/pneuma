use mio::Interest;
use std::{fmt::Write, io, net::SocketAddr, os::fd::AsRawFd};
use tokio::net::ToSocketAddrs;

use crate::{future::wait, reactor::Registration};

pub struct TcpStream {
    stream: mio::net::TcpStream,
    registration: Registration,
}

impl TcpStream {
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
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

        // If we hit an error while connecting return that error.

        let mut stream = mio::net::TcpStream::connect(addr)?;

        let reactor = pneuma::reactor::current();

        let registration =
            Registration::register(&mut stream, Interest::READABLE | Interest::WRITABLE)?;

        if let Ok(Some(err)) | Err(err) = stream.take_error() {
            return Err(err);
        }
        loop {
            // If we can get a peer address it means the stream is
            // connected.

            let Err(err) = stream.peer_addr() else {
                return Ok(TcpStream {
                    stream,
                    registration,
                });
            };

            // `NotConnected` (`ENOTCONN`) means the socket not yet
            // connected, but still working on it. `ECONNREFUSED` will
            // be reported if it fails.
            if err.kind() != io::ErrorKind::NotConnected
                && err.raw_os_error() != Some(libc::EINPROGRESS)
            {
                return Err(err);
            }
            // Socket is not (yet) connected but haven't hit an
            // error either. So we return yield and wait for
            // another event.
            pneuma::uthread::park()?;
        }
    }
}

impl io::Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        pneuma::reactor::op::write(&mut self.stream, &self.registration, buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
