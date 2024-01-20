use pneuma::thread::spawn;
use pneuma::time::{sleep, Duration, Instant};

#[test]
fn sleep_os_thread() {
    let start = Instant::now();
    sleep(Duration::from_millis(112)).unwrap();
    assert!(dbg!(start.elapsed().as_millis()) >= 112);
}

#[test]
fn sleep_green_thread() {
    spawn(|| {
        let start = Instant::now();
        sleep(Duration::from_millis(112)).unwrap();
        assert!(dbg!(start.elapsed().as_millis()) >= 112);
    })
    .join();
}

#[test]
fn sleep_concurrent() {
    let start = Instant::now();
    let mut handles = vec![];
    for i in 0..100 {
        let handle = spawn(move || {
            sleep(Duration::from_millis(50)).unwrap();
            println!("{i}");
            dbg!(start.elapsed());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join();
    }
    dbg!(start.elapsed());
    assert!(dbg!(start.elapsed().as_millis()) < 100);
}
