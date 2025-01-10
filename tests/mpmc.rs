use std::{
    sync::mpsc::{RecvTimeoutError, SendError, TryRecvError},
    thread::scope,
    time::{Duration, Instant},
};

use pneuma::{
    sync::mpmc::{channel, Receiver, Sender},
    uthread::{spawn, yield_now},
};

const ITERATIONS: usize = 100;

#[test]
fn mpmc_zero_cap_channel_sender() {
    let (sender, receiver) = pneuma::sync::mpmc::sync_channel(0);

    sender
        .try_send(0)
        .expect_err("there is no capacity on the channel.");

    pneuma::uthread::spawn(move || {
        for _ in 0..ITERATIONS {
            receiver.recv().expect("the sender is till open");
        }
    });

    for i in 0..ITERATIONS {
        sender.send(i).expect("the channel is still open");
    }

    sender.send(0).expect_err("the receiver is closed");
}

#[test]
fn mpmc_zero_cap_channel_receiver() {
    let (sender, receiver) = pneuma::sync::mpmc::sync_channel(0);

    receiver.try_recv().expect_err("no data set yet");

    pneuma::uthread::spawn(move || {
        for i in 0..ITERATIONS {
            sender.send(i).expect("the receiver is till open");
        }
    });

    for _ in 0..ITERATIONS {
        receiver.recv().expect("the channel is still open");
    }

    receiver.recv().expect_err("the sender is closed");
}

#[test]
fn mpmc_bounded_cap_channel_sender() {
    let (sender, receiver) = pneuma::sync::mpmc::sync_channel(1);

    sender.try_send(0).expect("there is capacity");
    sender.try_send(0).expect_err("there is no capacity left.");

    pneuma::uthread::spawn(move || {
        receiver.recv().unwrap();
        for _ in 0..ITERATIONS {
            receiver.recv().expect("the sender is till open");
        }
    });

    for i in 0..ITERATIONS {
        sender.send(i).expect("the channel is still open");
    }

    sender.send(0).expect_err("the receiver is closed");
}

#[test]
fn mpmc_bounded_cap_channel_receiver() {
    let (sender, receiver) = pneuma::sync::mpmc::sync_channel(1);

    receiver.try_recv().expect_err("no data set yet");

    pneuma::uthread::spawn(move || {
        for i in 0..ITERATIONS {
            sender.send(i).expect("the receiver is till open");
        }
    });

    for _ in 0..ITERATIONS {
        receiver.recv().expect("the channel is still open");
    }

    receiver.recv().expect_err("the sender is closed");
}

#[test]
fn mpmc_unbounded_cap_channel_sender() {
    let (sender, receiver) = pneuma::sync::mpmc::channel();

    sender.try_send(0).expect("there is infinite capacity");

    let handle = pneuma::uthread::spawn(move || {
        receiver.recv().unwrap();
        for _ in 0..ITERATIONS {
            receiver.recv().expect("the sender is till open");
        }
    });
    yield_now();
    for i in 0..ITERATIONS {
        sender.send(i).expect("the channel is still open");
    }
    handle.join();
    sender.send(0).expect_err("the receiver is closed");
}

#[test]
fn mpmc_unbounded_cap_channel_receiver() {
    let (sender, receiver) = pneuma::sync::mpmc::channel();

    receiver.try_recv().expect_err("no data set yet");

    let handle = pneuma::uthread::spawn(move || {
        for i in 0..ITERATIONS {
            sender.send(i).expect("the receiver is till open");
        }
    });

    for _ in 0..ITERATIONS {
        receiver.recv().expect("the channel is still open");
    }
    handle.join();

    receiver.recv().expect_err("the sender is closed");
}

#[test]
fn mpmc_iterator_receiver() {
    let (sender, receiver) = pneuma::sync::mpmc::sync_channel(1);

    let handle = pneuma::uthread::spawn(move || {
        for (i, j) in receiver.iter().enumerate() {
            assert_eq!(i, j);
        }
    });

    for i in 0..ITERATIONS {
        sender.send(i).unwrap();
    }
    drop(sender);
    handle.join();
}
