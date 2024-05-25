use pneuma::net;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

#[test]
fn tcp_connect_and_write() {
    let addr: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    let tcp_listener = std::net::TcpListener::bind(addr).unwrap();

    const MESSAGE: &[u8] = b"hello world";

    let thread = std::thread::spawn(move || {
        let mut buf = [0u8; MESSAGE.len()];
        tcp_listener
            .accept()
            .unwrap()
            .0
            .read_exact(&mut buf)
            .unwrap();
        assert_eq!(buf, MESSAGE);
    });

    let mut socket = pneuma::net::TcpStream::connect(addr).unwrap();

    socket.write_all(MESSAGE).unwrap();

    thread.join().unwrap();
}
