pub mod x86_64;
pub use x86_64::*;

pub mod config;
pub use config::{BootSourceConfig, InitrdConfig};

/// Default (smallest) memory page size for the supported architectures.
pub const PAGE_SIZE: usize = 4096;

pub const DEFAULT_KERNEL_CMDLINE: &str =
    "console=ttyS0 noapic noacpi reboot=k panic=1 pci=off nomodule";
