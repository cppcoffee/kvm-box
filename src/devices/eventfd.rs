use std::io;
use std::ops::Deref;

use vm_superio::Trigger;
use vmm_sys_util::eventfd::{EventFd, EFD_NONBLOCK};

/// Wrapper for implementing the trigger functionality for `EventFd`.
///
/// The trigger is used for handling events in the legacy devices.
#[derive(Debug)]
pub struct EventFdTrigger(pub EventFd);

impl Trigger for EventFdTrigger {
    type E = io::Error;

    fn trigger(&self) -> io::Result<()> {
        self.write(1)
    }
}

impl Deref for EventFdTrigger {
    type Target = EventFd;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EventFdTrigger {
    /// Create an `EventFdTrigger`.
    pub fn new() -> Self {
        let event_fd = EventFd::new(EFD_NONBLOCK).expect("Cannot create eventfd");
        Self(event_fd)
    }

    /// Clone an `EventFdTrigger`.
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

}
