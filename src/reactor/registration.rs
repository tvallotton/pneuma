use std::{io, mem::transmute, sync::atomic::AtomicU8};

use mio::{Interest, Token};

use std::sync::atomic::Ordering::Relaxed;

pub struct Registration {
    key: usize,
    interests: Interest,
}

impl Registration {
    pub fn register<S>(source: &mut S, interests: mio::Interest) -> io::Result<Registration>
    where
        S: mio::event::Source + ?Sized,
    {
        let uthread = pneuma::uthread::current();
        let mut reactor = pneuma::reactor::current().reactor.lock().unwrap();

        let key = reactor.tokens.insert(uthread);

        let registration = Registration { key, interests };

        reactor
            .poll
            .registry()
            .register(source, mio::Token(key), interests)
            .map(|_| registration)
    }

    pub fn reregister<S>(&mut self, source: &mut S, interests: mio::Interest) -> io::Result<()>
    where
        S: mio::event::Source + ?Sized,
    {
        let reactor = pneuma::reactor::current().reactor.lock().unwrap();

        reactor
            .poll
            .registry()
            .reregister(source, Token(self.key), interests)?;

        self.interests = interests;

        Ok(())
    }

    pub fn interests(&self) -> Interest {
        self.interests
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        pneuma::runtime::current()
            .reactor
            .reactor
            .lock()
            .unwrap()
            .tokens
            .remove(self.key);
    }
}
