use pneuma::net::{TcpListener, TcpStream};

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
