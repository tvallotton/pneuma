pub fn yield_now() {
    dbg!("before unpark");
    pneuma::uthread::current().unpark();
    dbg!("after unpark, before park");
    pneuma::uthread::park().unwrap();
    dbg!("after yield");
}
