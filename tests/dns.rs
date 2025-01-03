use pneuma::net::ToSocketAddrs;

#[test]
fn wikipedia() {
    "www.wikipedia.com:80"
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
}

#[test]
fn localhost() {
    let localhost = "localhost:80".to_socket_addrs().unwrap().next().unwrap();
    let ip127001 = "127.0.0.1:80".to_socket_addrs().unwrap().next().unwrap();
    assert_eq!(localhost, ip127001)
}
