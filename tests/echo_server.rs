use pneuma::net::{TcpListener, TcpStream};
use pneuma::uthread;
use std::io::{Read, Write};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512]; // Buffer to store incoming data

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed by client
            Ok(n) => {
                // Echo the data back to the client
                if let Err(e) = stream.write_all(&buffer[0..n]) {
                    eprintln!("Failed to write to stream: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
                break;
            }
        }
    }

    println!("Connection closed");
}

fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?; // Bind to localhost on port 8080

    for stream in listener.incoming().take(1) {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                uthread::spawn(|| handle_client(stream)); // Handle each connection in a new thread
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn run_client(message: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;

    write!(&mut stream, "{message}")?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer)?;

    assert_eq!(&buffer[..n], message.as_bytes());

    Ok(())
}

#[test]
fn echo_server() {
    pneuma::uthread::scope(|s| {
        s.spawn(run_server);
        s.spawn(|| run_client("hello my world"));
    })
}
