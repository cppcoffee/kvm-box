use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result};
use linux_loader::configurator::linux::LinuxBootConfigurator;
use linux_loader::configurator::{BootConfigurator, BootParams};
use linux_loader::loader::load_cmdline;
use linux_loader::{
    bootparam::boot_params,
    loader::{elf::Elf, KernelLoader},
};
use vm_memory::{
    Address, GuestAddress, GuestMemory, GuestMemoryMmap, GuestMemoryRegion, ReadVolatile,
};

// Value taken from https://elixir.bootlin.com/linux/v5.10.68/source/arch/x86/include/uapi/asm/e820.h#L31
// Usable normal RAM
const E820_RAM: u32 = 1;
// Reserved area that should be avoided during memory allocations
const E820_RESERVED: u32 = 2;

// EBDA is located in the last 1 KiB of the first 640KiB of memory, i.e in the range:
// [0x9FC00, 0x9FFFF]
// We mark first [0x0, EBDA_START] region as usable RAM
// and [EBDA_START, (EBDA_START + EBDA_SIZE)] as reserved.
const EBDA_START: u64 = 0x9fc00;
const EBDA_SIZE: u64 = 1 << 10;
const FIRST_ADDR_PAST_32BITS: u64 = 1 << 32;

/// Size of MMIO gap at top of 32-bit address space.
pub const MEM_32BIT_GAP_SIZE: u64 = 768 << 20;
/// The start of the memory area reserved for MMIO devices.
pub const MMIO_MEM_START: u64 = FIRST_ADDR_PAST_32BITS - MEM_32BIT_GAP_SIZE;
// TODO: The size of the memory area reserved for MMIO devices.
//pub const MMIO_MEM_SIZE: u64 = MEM_32BIT_GAP_SIZE;

pub fn load_kernel<P: AsRef<Path>>(
    kernel_image_path: P,
    guest_mem: &GuestMemoryMmap,
) -> Result<GuestAddress> {
    let mut kernel_file = File::open(kernel_image_path).expect("open kernel file failed");
    let kernel_load = Elf::load(
        guest_mem,
        None,
        &mut kernel_file,
        Some(GuestAddress(super::layout::KERNEL_START_ADDRESS)),
    )?
    .kernel_load;

    Ok(kernel_load)
}

pub fn load_initrd<P: AsRef<Path>>(
    initrd_path: P,
    vm_memory: &GuestMemoryMmap,
) -> Result<crate::arch::InitrdConfig> {
    let mut image = File::open(initrd_path).context("failed to open initrd file")?;

    // Get the image size
    let size = image.seek(SeekFrom::End(0))? as usize;
    if size == 0 {
        anyhow::bail!("Initrd image seek returned a size of zero")
    }

    // Go back to the image start
    image.seek(SeekFrom::Start(0))?;

    // Get the target address
    let address = initrd_load_addr(vm_memory, size as usize)?;

    // Load the image into memory
    let mut slice = vm_memory.get_slice(GuestAddress(address), size as usize)?;

    image.read_exact_volatile(&mut slice)?;

    Ok(crate::arch::InitrdConfig {
        address: GuestAddress(address),
        size,
    })
}

/// Returns the memory address where the initrd could be loaded.
pub fn initrd_load_addr(vm_memory: &GuestMemoryMmap, initrd_size: usize) -> Result<u64> {
    let first_region = vm_memory
        .find_region(GuestAddress::new(0))
        .context("failed to find guest memory region")?;

    let lowmem_size = first_region.len() as usize;

    if lowmem_size < initrd_size {
        anyhow::bail!("initrd size is too big")
    }

    let align_to_pagesize = |address| address & !(crate::arch::PAGE_SIZE - 1);
    Ok(align_to_pagesize(lowmem_size - initrd_size) as u64)
}

pub fn configure_system(
    guest_mem: &GuestMemoryMmap,
    cmdline_addr: GuestAddress,
    cmdline_size: usize,
    initrd: &Option<crate::arch::InitrdConfig>,
) -> Result<()> {
    const KERNEL_BOOT_FLAG_MAGIC: u16 = 0xaa55;
    const KERNEL_HDR_MAGIC: u32 = 0x5372_6448;
    const KERNEL_LOADER_OTHER: u8 = 0xff;
    const KERNEL_MIN_ALIGNMENT_BYTES: u32 = 0x0100_0000; // Must be non-zero.

    let first_addr_past_32bits = GuestAddress(FIRST_ADDR_PAST_32BITS);
    let end_32bit_gap_start = GuestAddress(MMIO_MEM_START);

    let himem_start = GuestAddress(crate::arch::layout::KERNEL_START_ADDRESS);

    // TODO: support mptable
    // Note that this puts the mptable at the last 1k of Linux's 640k base RAM
    //mptable::setup_mptable(guest_mem, num_cpus)?;

    let mut params = boot_params::default();

    params.hdr.type_of_loader = KERNEL_LOADER_OTHER;
    params.hdr.boot_flag = KERNEL_BOOT_FLAG_MAGIC;
    params.hdr.header = KERNEL_HDR_MAGIC;
    params.hdr.cmd_line_ptr = u32::try_from(cmdline_addr.raw_value())?;
    params.hdr.cmdline_size = u32::try_from(cmdline_size)?;
    params.hdr.kernel_alignment = KERNEL_MIN_ALIGNMENT_BYTES;

    if let Some(initrd_config) = initrd {
        params.hdr.ramdisk_image = u32::try_from(initrd_config.address.raw_value())?;
        params.hdr.ramdisk_size = u32::try_from(initrd_config.size)?;
    }

    add_e820_entry(&mut params, 0, EBDA_START, E820_RAM)?;
    add_e820_entry(&mut params, EBDA_START, EBDA_SIZE, E820_RESERVED)?;

    let last_addr = guest_mem.last_addr();
    if last_addr < end_32bit_gap_start {
        add_e820_entry(
            &mut params,
            himem_start.raw_value(),
            // it's safe to use unchecked_offset_from because
            // mem_end > himem_start
            last_addr.unchecked_offset_from(himem_start) + 1,
            E820_RAM,
        )?;
    } else {
        add_e820_entry(
            &mut params,
            himem_start.raw_value(),
            // it's safe to use unchecked_offset_from because
            // end_32bit_gap_start > himem_start
            end_32bit_gap_start.unchecked_offset_from(himem_start),
            E820_RAM,
        )?;

        if last_addr > first_addr_past_32bits {
            add_e820_entry(
                &mut params,
                first_addr_past_32bits.raw_value(),
                // it's safe to use unchecked_offset_from because
                // mem_end > first_addr_past_32bits
                last_addr.unchecked_offset_from(first_addr_past_32bits) + 1,
                E820_RAM,
            )?;
        }
    }

    LinuxBootConfigurator::write_bootparams(
        &BootParams::new(&params, GuestAddress(crate::arch::layout::ZERO_PAGE_START)),
        guest_mem,
    )
    .context("failed to write bootparams")?;

    Ok(())
}

pub fn load_boot_cmdline(
    guest_mem: &GuestMemoryMmap,
    boot_source_cfg: &crate::arch::BootSourceConfig,
) -> Result<(GuestAddress, usize)> {
    let cmdline_addr = GuestAddress(crate::arch::layout::CMDLINE_START);

    let (boot_cmdline, cmdline_size) = boot_source_cfg
        .to_kernel_cmdline()
        .context("failed to build kernel cmdline")?;

    load_cmdline(guest_mem, cmdline_addr, &boot_cmdline).context("failed to load boot cmdline")?;

    Ok((cmdline_addr, cmdline_size))
}

/// Add an e820 region to the e820 map.
/// Returns Ok(()) if successful, or an error if there is no space left in the map.
fn add_e820_entry(params: &mut boot_params, addr: u64, size: u64, mem_type: u32) -> Result<()> {
    if params.e820_entries as usize >= params.e820_table.len() {
        anyhow::bail!("e820 configuration error")
    }

    params.e820_table[params.e820_entries as usize].addr = addr;
    params.e820_table[params.e820_entries as usize].size = size;
    params.e820_table[params.e820_entries as usize].type_ = mem_type;
    params.e820_entries += 1;

    Ok(())
}
