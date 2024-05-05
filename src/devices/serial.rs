use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use vm_superio::serial::SerialEvents;
use vm_superio::{Serial, Trigger};

use super::{BusDevice, EventFdTrigger};

/// Sets up the serial device.
pub fn setup_serial_device(
    input: std::io::Stdin,
    out: std::io::Stdout,
) -> Result<Arc<Mutex<BusDevice>>> {
    let interrupt_evt = EventFdTrigger::new();

    let serial = Arc::new(Mutex::new(BusDevice::Serial(SerialWrapper {
        serial: Serial::with_events(interrupt_evt, SerialEventsWrapper, SerialOut::Stdout(out)),
        input: Some(input),
    })));

    Ok(serial)
}

#[derive(Debug)]
pub enum SerialOut {
    Sink(std::io::Sink),
    Stdout(std::io::Stdout),
}

impl Write for SerialOut {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Sink(sink) => sink.write(buf),
            Self::Stdout(stdout) => stdout.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Sink(sink) => sink.flush(),
            Self::Stdout(stdout) => stdout.flush(),
        }
    }
}

/// Wrapper over the imported serial device.
#[derive(Debug)]
pub struct SerialWrapper<T: Trigger, EV: SerialEvents, I: Read + AsRawFd + Send> {
    /// Serial device object.
    pub serial: Serial<T, EV, SerialOut>,
    /// Input to the serial device (needs to be readable).
    pub input: Option<I>,
}

#[derive(Debug)]
pub struct SerialEventsWrapper;

impl SerialEvents for SerialEventsWrapper {
    fn buffer_read(&self) {}

    fn out_byte(&self) {}

    fn tx_lost_byte(&self) {}

    fn in_buffer_empty(&self) {}
}

/// Type for representing a serial device.
pub type SerialDevice<I> = SerialWrapper<EventFdTrigger, SerialEventsWrapper, I>;

impl<I: Read + AsRawFd + Send + std::fmt::Debug + 'static>
    SerialWrapper<EventFdTrigger, SerialEventsWrapper, I>
{
    pub fn bus_read(&mut self, offset: u64, data: &mut [u8]) {
        if let (Ok(offset), 1) = (u8::try_from(offset), data.len()) {
            data[0] = self.serial.read(offset);
        }
    }

    pub fn bus_write(&mut self, offset: u64, data: &[u8]) {
        if let (Ok(offset), 1) = (u8::try_from(offset), data.len()) {
            if let Err(err) = self.serial.write(offset, data[0]) {
                log::error!("Failed the write to serial: {:?}", err);
            }
        }
    }
}
