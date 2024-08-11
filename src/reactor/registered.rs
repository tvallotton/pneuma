use std::{io, mem::transmute};

use mio::{event::Source, Interest, Token};

use pneuma::{uthread::UThread, utils::IgnorePoison};

pub struct Registered<S: Source> {
    pub source: S,
    uthread: usize,
    interests: Interest,
}

impl<S> Registered<S>
where
    S: mio::event::Source,
{
    pub fn register(source: S, interests: mio::Interest) -> io::Result<Registered<S>> {
        let uthread = pneuma::uthread::current();

        let uthread: usize = unsafe { transmute(uthread) };
        let mut registration = Registered {
            source,
            uthread,
            interests,
        };

        let res = pneuma::reactor::current()
            .mio
            .lock()
            .ignore_poison()
            .poll
            .registry()
            .register(&mut registration.source, mio::Token(uthread), interests)
            .map(|_| registration);

        res
    }

    pub fn reregister(&mut self, interests: mio::Interest) -> io::Result<()> {
        let uthread: usize = unsafe { transmute(pneuma::uthread::current()) };
        dbg!("lock");

        pneuma::reactor::current()
            .mio
            .lock()
            .ignore_poison()
            .poll
            .registry()
            .reregister(&mut self.source, Token(uthread), interests)?;

        self.interests = interests;
        dbg!("release");
        Ok(())
    }

    pub fn interests(&self) -> Interest {
        self.interests
    }

    pub fn add(&mut self, interest: Interest) -> io::Result<()> {
        let interests_changed = self.interests | interest != self.interests;
        let thread_changed =
            self.uthread != *unsafe { transmute::<&UThread, &usize>(&pneuma::uthread::current()) };
        if interests_changed || thread_changed {
            self.reregister(self.interests | interest)?;
        }
        Ok(())
    }
}

impl<S: Source> Drop for Registered<S> {
    fn drop(&mut self) {
        pneuma::reactor::current()
            .mio
            .lock()
            .ignore_poison()
            .poll
            .registry()
            .deregister(&mut self.source)
            .ok()
            .map(|_| unsafe {
                transmute::<usize, UThread>(self.uthread);
            });
    }
}
