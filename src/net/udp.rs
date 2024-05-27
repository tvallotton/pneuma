use std::io::{self, ErrorKind};

use crate::reactor::Registration;
use mio::Interest;
use std::net::SocketAddr;

pub struct UdpSocket {
    stream: mio::net::UdpSocket,
    registration: Registration,
}

impl UdpSocket {
    fn bind(addr: SocketAddr) -> io::Result<UdpSocket> {
        let mut stream = mio::net::UdpSocket::bind(addr)?;

        let registration =
            Registration::register(&mut stream, Interest::READABLE | Interest::WRITABLE)?;

        Ok(UdpSocket {
            stream,
            registration,
        })
    }

    fn connect(&mut self, addr: SocketAddr) -> io::Result<()> {
        loop {
            let Err(err) = self.stream.connect(addr) else {
                return Ok(());
            };
            if err.kind() == ErrorKind::WouldBlock {
                pneuma::uthread::park();
            }
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    // fn send_to(&s)
}
