use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use kvm_ioctls::VmFd;
use vm_superio::Serial;
use vmm_sys_util::eventfd::{EventFd, EFD_NONBLOCK};

use crate::devices::{BusDevice, EventFdTrigger, SerialDevice, SerialEventsWrapper, SerialOut};

/// The `PortIODeviceManager` is a wrapper that is used for registering legacy devices
/// on an I/O Bus. It currently manages the uart and i8042 devices.
#[derive(Debug)]
pub struct PortIODeviceManager {
    pub io_bus: crate::devices::Bus,
    // BusDevice::Serial
    pub stdio_serial: Arc<Mutex<BusDevice>>,

    // Communication event on ports 1 & 3.
    pub com_evt_1_3: EventFdTrigger,
    // Communication event on ports 2 & 4.
    pub com_evt_2_4: EventFdTrigger,
    // Keyboard event.
    pub kbd_evt: EventFd,
}

impl PortIODeviceManager {
    /// x86 global system interrupt for communication events on serial ports 1
    /// & 3. See
    /// <https://en.wikipedia.org/wiki/Interrupt_request_(PC_architecture)>.
    const COM_EVT_1_3_GSI: u32 = 4;
    /// x86 global system interrupt for communication events on serial ports 2
    /// & 4. See
    /// <https://en.wikipedia.org/wiki/Interrupt_request_(PC_architecture)>.
    const COM_EVT_2_4_GSI: u32 = 3;
    /// x86 global system interrupt for keyboard port.
    /// See <https://en.wikipedia.org/wiki/Interrupt_request_(PC_architecture)>.
    const KBD_EVT_GSI: u32 = 1;
    /// Legacy serial port device addresses. See
    /// <https://tldp.org/HOWTO/Serial-HOWTO-10.html#ss10.1>.
    const SERIAL_PORT_ADDRESSES: [u64; 4] = [0x3f8, 0x2f8, 0x3e8, 0x2e8];
    /// Size of legacy serial ports.
    const SERIAL_PORT_SIZE: u64 = 0x8;

    /// Create a new DeviceManager handling legacy devices (uart, i8042).
    pub fn new(serial: Arc<Mutex<BusDevice>>) -> Result<Self> {
        debug_assert!(matches!(*serial.lock().unwrap(), BusDevice::Serial(_)));
        let io_bus = crate::devices::Bus::new();
        let com_evt_1_3 = serial
            .lock()
            .expect("Poisoned lock")
            .serial_mut()
            .unwrap()
            .serial
            .interrupt_evt()
            .try_clone()?;

        let com_evt_2_4 = EventFdTrigger::new();
        let kbd_evt = EventFd::new(EFD_NONBLOCK)?;

        Ok(PortIODeviceManager {
            io_bus,
            stdio_serial: serial,
            com_evt_1_3,
            com_evt_2_4,
            kbd_evt,
        })
    }

    /// Register supported legacy devices.
    pub fn register_devices(&mut self, vm_fd: &VmFd) -> Result<()> {
        let serial_2_4 = Arc::new(Mutex::new(BusDevice::Serial(SerialDevice {
            serial: Serial::with_events(
                self.com_evt_2_4.try_clone()?.try_clone()?,
                SerialEventsWrapper,
                SerialOut::Sink(std::io::sink()),
            ),
            input: None,
        })));

        let serial_1_3 = Arc::new(Mutex::new(BusDevice::Serial(SerialDevice {
            serial: Serial::with_events(
                self.com_evt_1_3.try_clone()?.try_clone()?,
                SerialEventsWrapper,
                SerialOut::Sink(std::io::sink()),
            ),
            input: None,
        })));

        self.io_bus.insert(
            self.stdio_serial.clone(),
            Self::SERIAL_PORT_ADDRESSES[0],
            Self::SERIAL_PORT_SIZE,
        )?;
        self.io_bus.insert(
            serial_2_4.clone(),
            Self::SERIAL_PORT_ADDRESSES[1],
            Self::SERIAL_PORT_SIZE,
        )?;
        self.io_bus.insert(
            serial_1_3,
            Self::SERIAL_PORT_ADDRESSES[2],
            Self::SERIAL_PORT_SIZE,
        )?;
        self.io_bus.insert(
            serial_2_4,
            Self::SERIAL_PORT_ADDRESSES[3],
            Self::SERIAL_PORT_SIZE,
        )?;

        vm_fd.register_irqfd(&self.com_evt_1_3, Self::COM_EVT_1_3_GSI)?;
        vm_fd.register_irqfd(&self.com_evt_2_4, Self::COM_EVT_2_4_GSI)?;
        vm_fd.register_irqfd(&self.kbd_evt, Self::KBD_EVT_GSI)?;

        Ok(())
    }
}
