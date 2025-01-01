use pneuma::future::wait;
use pneuma::uthread::spawn;
use std::future::{poll_fn, Future};
use std::io::Read;
use std::task::Poll;

// a future that returns pending once.
fn yield_now() -> impl Future<Output = ()> + Unpin {
    let mut yielded = false;
    poll_fn(move |cx| {
        if !yielded {
            yielded = true;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        Poll::Ready(())
    })
}

#[test]
pub fn smoke_test() {
    let out = wait(async {
        yield_now().await;
        10
    });
    assert_eq!(out, 10);
}

// #[test]
// pub fn tokio() {
//     spawn(|| {
//         wait(async {
//             use tokio::io::*;
//             let listener = tokio::net::TcpListener::bind("127.0.0.1:10001")
//                 .await
//                 .unwrap();

//             let mut stream = listener.accept().await.unwrap().0;
//             let mut buf = [0; 20];
//             let read = stream.read(&mut buf).await.unwrap();
//             assert_eq!(&buf[..read], b"foo bar");
//         });
//     });
//     spawn(|| {
//         use std::io::Write;
//         let mut stream = pneuma::net::TcpStream::connect("127.0.0.1:10001").unwrap();

//         writeln!(stream, "foo bar").unwrap();
//     });
// }
