//! Signal driver

use crate::io::driver::Driver as IoDriver;
use crate::io::Registration;
use crate::park::Park;
use crate::runtime::context;
use crate::signal::registry::globals;
use mio_uds::UnixStream;
use std::io::{self, Read};
use std::sync::{Arc, Weak};
use std::time::Duration;

/// Responsible for registering wakeups when an OS signal is received, and
/// subsequently dispatching notifications to any signal listeners as appropriate.
///
/// Note: this driver relies on having an enabled IO driver in order to listen to
/// pipe write wakeups.
#[derive(Debug)]
pub(crate) struct Driver {
    /// Thread parker. The `Driver` park implementation delegates to this.
    park: IoDriver,

    /// A pipe for receiving wake events from the signal handler
    receiver: UnixStream,

    /// The actual registraiton for `receiver` when active.
    /// Lazily bound at the first signal registration.
    registration: Registration,

    /// Shared state
    inner: Arc<Inner>,
}

#[derive(Clone, Debug)]
pub(crate) struct Handle {
    inner: Weak<Inner>,
}

#[derive(Debug)]
pub(super) struct Inner(());

// ===== impl Driver =====

impl Driver {
    /// Creates a new signal `Driver` instance that delegates wakeups to `park`.
    pub(crate) fn new(park: IoDriver) -> io::Result<Self> {
        // NB: We give each driver a "fresh" reciever file descriptor to avoid
        // the issues described in alexcrichton/tokio-process#42.
        //
        // In the past we would reuse the actual receiver file descriptor and
        // swallow any errors around double registration of the same descriptor.
        // I'm not sure if the second (failed) registration simply doesn't end up
        // receiving wake up notifications, or there could be some race condition
        // when consuming readiness events, but having distinct descriptors for
        // distinct PollEvented instances appears to mitigate this.
        //
        // Unfortunately we cannot just use a single global PollEvented instance
        // either, since we can't compare Handles or assume they will always
        // point to the exact same reactor.
        let receiver = globals().receiver.try_clone()?;
        let registration =
            Registration::new_with_ready_and_handle(&receiver, mio::Ready::all(), park.handle())?;

        Ok(Self {
            park,
            receiver,
            registration,
            inner: Arc::new(Inner(())),
        })
    }

    /// Returns a handle to this event loop which can be sent across threads
    /// and can be used as a proxy to the event loop itself.
    pub(crate) fn handle(&self) -> Handle {
        Handle {
            inner: Arc::downgrade(&self.inner),
        }
    }

    fn process(&self) {
        // Check if the pipe is ready to read and therefore has "woken" us up
        match self.registration.take_read_ready() {
            Ok(Some(ready)) => assert!(ready.is_readable()),
            Ok(None) => return, // No wake has arrived, bail
            Err(e) => panic!("reactor gone: {}", e),
        }

        // Drain the pipe completely so we can receive a new readiness event
        // if another signal has come in.
        let mut buf = [0; 128];
        loop {
            match (&self.receiver).read(&mut buf) {
                Ok(0) => panic!("EOF on self-pipe"),
                Ok(_) => continue, // Keep reading
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("Bad read on self-pipe: {}", e),
            }
        }

        // Broadcast any signals which were received
        globals().broadcast();
    }
}

// ===== impl Park for Driver =====

impl Park for Driver {
    type Unpark = <IoDriver as Park>::Unpark;
    type Error = io::Error;

    fn unpark(&self) -> Self::Unpark {
        self.park.unpark()
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.park.park()?;
        self.process();
        Ok(())
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        self.park.park_timeout(duration)?;
        self.process();
        Ok(())
    }

    fn shutdown(&mut self) {
        self.park.shutdown()
    }
}

// ===== impl Handle =====

impl Handle {
    /// Returns a handle to the current driver
    ///
    /// # Panics
    ///
    /// This function panics if there is no current signal driver set.
    pub(super) fn current() -> Self {
        context::signal_handle().expect(
            "there is no signal driver running, must be called from the context of Tokio runtime",
        )
    }

    pub(super) fn check_inner(&self) -> io::Result<()> {
        if self.inner.strong_count() > 0 {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "signal driver gone"))
        }
    }
}
