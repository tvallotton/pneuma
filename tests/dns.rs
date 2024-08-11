use pneuma::net::ToSocketAddrs;

#[test]
fn wikipedia() {
    let ip = "www.wikipedia.com:80"
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    dbg!(ip);
}

#[test]
fn localhost() {
    let localhost = "localhost:80".to_socket_addrs().unwrap().next().unwrap();
    let ip127001 = "127.0.0.1:80".to_socket_addrs().unwrap().next().unwrap();
    assert_eq!(localhost, ip127001)
}
