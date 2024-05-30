use pneuma::net::TcpStream;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::time::Duration;

#[test]
fn tcp_connect_read_and_write() {
    let addr: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    let tcp_listener = std::net::TcpListener::bind(addr).unwrap();

    const MESSAGE: &[u8] = b"hello world!";
    const RESPONSE: &[u8] = b"good bye world!";

    let thread = std::thread::spawn(move || {
        let mut buf = [0u8; MESSAGE.len()];
        let mut stream = tcp_listener.accept().unwrap().0;

        stream.read_exact(&mut buf).unwrap();
        assert_eq!(buf, MESSAGE);

        stream.write_all(RESPONSE).unwrap();
    });

    let mut buf = [0u8; RESPONSE.len()];
    let mut socket = pneuma::net::TcpStream::connect(addr).unwrap();

    socket.write_all(MESSAGE).unwrap();

    socket.read_exact(&mut buf).unwrap();

    assert_eq!(buf, RESPONSE);

    thread.join().unwrap();
}

#[test]
fn tcp_connection_refused() {
    let addr: SocketAddr = "127.0.0.1:8082".parse().unwrap();
    let err = TcpStream::connect(addr).unwrap_err().kind();
    assert_eq!(err, ErrorKind::ConnectionRefused);
}

#[test]
fn tcp_connect_timout() {
    let addr: SocketAddr = "127.0.0.1:8083".parse().unwrap();
    let _listener = TcpListener::bind(addr);

    // We create 129 because the backlog is 128 by default
    let streams: Vec<_> = (0..512)
        .map(|_| TcpStream::connect_timeout(addr, Duration::from_nanos(100)))
        .collect();

    let err = streams.iter().last().unwrap().as_ref().unwrap_err().kind();
    assert_eq!(err, ErrorKind::TimedOut)
}
