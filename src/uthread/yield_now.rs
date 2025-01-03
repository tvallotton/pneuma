pub fn yield_now() {
    pneuma::uthread::current().unpark();
    pneuma::uthread::park();
}
