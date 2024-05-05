/// Initial stack for the boot CPU.
pub const BOOT_STACK_POINTER: u64 = 0x8ff0;

/// Address for the TSS setup.
pub const KVM_TSS_ADDRESS: usize = 0xfffb_d000;

/// Start of the high memory.
pub const KERNEL_START_ADDRESS: u64 = 0x0010_0000; // 1 MB.

/// The 'zero page', a.k.a linux kernel bootparams.
pub const ZERO_PAGE_START: u64 = 0x7000;

/// Kernel command line start address.
pub const CMDLINE_START: u64 = 0x20000;
/// Kernel command line maximum size.
pub const CMDLINE_MAX_SIZE: usize = 2048;
