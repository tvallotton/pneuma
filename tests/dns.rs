use pneuma::net::{lookup, ToSocketAddrs};

#[test]
fn wikipedia() {
    let addr: Vec<_> = lookup("www.wikipedia.com").unwrap().collect();
    dbg!(addr);
}
