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
