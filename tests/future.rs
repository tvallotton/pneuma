use pneuma::future::wait;

#[test]
pub fn smoke_test() {
    let out = wait(async {
        tokio::task::yield_now().await;
        10
    });
    assert_eq!(out, 10);
}
