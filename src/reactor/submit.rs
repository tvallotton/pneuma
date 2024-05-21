use mio::Interest;

pub fn submit<F>(f: F, interest: Interest)
where
    F: FnOnce(),
{
    let rt = pneuma::runtime::current();
    let reactor = rt.reactor.reactor.lock().unwrap();
}
