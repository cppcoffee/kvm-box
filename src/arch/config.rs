use anyhow::Result;
use linux_loader::cmdline::Cmdline;

/// Strongly typed data structure used to configure the boot source of the
/// microvm.
#[derive(Debug, Default)]
pub struct BootSourceConfig {
    /// Path of the kernel image.
    pub kernel_image_path: String,
    /// Path of the initrd, if there is one.
    pub initrd_path: Option<String>,
    /// The boot arguments to pass to the kernel. If this field is uninitialized,
    /// DEFAULT_KERNEL_CMDLINE is used.
    pub boot_args: Option<String>,
}

impl BootSourceConfig {
    pub fn to_kernel_cmdline(&self) -> Result<(Cmdline, usize)> {
        let cmdline_str = match self.boot_args.as_ref() {
            None => super::DEFAULT_KERNEL_CMDLINE,
            Some(str) => str.as_str(),
        };

        let cmdline = Cmdline::try_from(cmdline_str, super::layout::CMDLINE_MAX_SIZE)?;

        let size = cmdline
            .as_cstring()
            .map(|cmdline_cstring| cmdline_cstring.as_bytes_with_nul().len())?;

        Ok((cmdline, size))
    }
}

/// Type for passing information about the initrd in the guest memory.
#[derive(Debug)]
pub struct InitrdConfig {
    /// Load address of initrd in guest memory
    pub address: vm_memory::GuestAddress,
    /// Size of initrd in guest memory
    pub size: usize,
}
