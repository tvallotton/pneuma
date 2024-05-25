use std::{io, mem::transmute, sync::atomic::AtomicU8};

use mio::{Interest, Token};

use std::sync::atomic::Ordering::Relaxed;

pub struct Registration {
    key: usize,
    interests: AtomicU8,
}

impl Registration {
    fn new(key: usize, interests: mio::Interest) -> Registration {
        let interests = AtomicU8::new(unsafe { transmute(interests) });
        Registration { key, interests }
    }

    pub fn register<S>(source: &mut S, interests: mio::Interest) -> io::Result<Registration>
    where
        S: mio::event::Source + ?Sized,
    {
        let uthread = pneuma::uthread::current();
        let mut reactor = pneuma::reactor::current().reactor.lock().unwrap();

        let key = reactor.tokens.insert(uthread);

        let registration = Registration::new(key, interests);

        reactor
            .poll
            .registry()
            .register(source, mio::Token(key), interests)
            .map(|_| registration)
    }

    pub fn reregister<S>(&self, source: &mut S, interests: mio::Interest) -> io::Result<()>
    where
        S: mio::event::Source + ?Sized,
    {
        let reactor = pneuma::reactor::current().reactor.lock().unwrap();

        reactor
            .poll
            .registry()
            .reregister(source, Token(self.key), interests)?;

        self.interests
            .store(unsafe { transmute(interests) }, Relaxed);
        Ok(())
    }

    pub fn interests(&self) -> Interest {
        unsafe { transmute(self.interests.load(Relaxed)) }
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
