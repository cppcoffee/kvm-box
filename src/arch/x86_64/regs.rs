use anyhow::{Context, Result};
use kvm_bindings::{kvm_fpu, kvm_regs, kvm_sregs};
use kvm_ioctls::VcpuFd;
use vm_memory::{Address, Bytes, GuestAddress, GuestMemory, GuestMemoryMmap};

use crate::arch::gdt::{gdt_entry, kvm_segment_from_gdt};

// Initial pagetables.
const PML4_START: u64 = 0x9000;
const PDPTE_START: u64 = 0xa000;
const PDE_START: u64 = 0xb000;

const BOOT_GDT_MAX: usize = 4;

const BOOT_GDT_OFFSET: u64 = 0x500;
const BOOT_IDT_OFFSET: u64 = 0x520;

const EFER_LMA: u64 = 0x400;
const EFER_LME: u64 = 0x100;

const X86_CR0_PE: u64 = 0x1;
const X86_CR0_PG: u64 = 0x8000_0000;
const X86_CR4_PAE: u64 = 0x20;

pub fn init_regs(vcpu: &VcpuFd, boot_ip: u64) -> Result<()> {
    let regs = kvm_regs {
        rflags: 2,
        rip: boot_ip,
        // Frame pointer. It gets a snapshot of the stack pointer (rsp) so that when adjustments are
        // made to rsp (i.e. reserving space for local variables or pushing values on to the stack),
        // local variables and function parameters are still accessible from a constant offset from
        // rbp.
        rsp: super::layout::BOOT_STACK_POINTER,
        // Starting stack pointer.
        rbp: super::layout::BOOT_STACK_POINTER,
        // Must point to zero page address per Linux ABI. This is x86_64 specific.
        rsi: super::layout::ZERO_PAGE_START,
        ..Default::default()
    };

    vcpu.set_regs(&regs).context("failed to set regs")?;

    Ok(())
}

// Configure Floating-Point Unit (FPU) registers for a given CPU.
pub fn init_fpu(vcpu: &VcpuFd) -> Result<()> {
    let fpu = kvm_fpu {
        fcw: 0x37f,
        mxcsr: 0x1f80,
        ..Default::default()
    };

    vcpu.set_fpu(&fpu).context("failed to set fpu")?;

    Ok(())
}

pub fn init_sregs(guest_mem: &GuestMemoryMmap, vcpu: &VcpuFd) -> Result<()> {
    let mut sregs = vcpu.get_sregs().context("failed to get sregs")?;

    configure_segments_and_sregs(guest_mem, &mut sregs)
        .context("failed to configure segments and sregs")?;

    setup_page_tables(guest_mem, &mut sregs).context("failed to setup page tables")?;

    vcpu.set_sregs(&sregs).context("failed to set sregs")?;

    Ok(())
}

fn configure_segments_and_sregs(guest_mem: &GuestMemoryMmap, sregs: &mut kvm_sregs) -> Result<()> {
    let gdt_table: [u64; BOOT_GDT_MAX] = [
        gdt_entry(0, 0, 0),            // NULL
        gdt_entry(0xa09b, 0, 0xfffff), // CODE
        gdt_entry(0xc093, 0, 0xfffff), // DATA
        gdt_entry(0x808b, 0, 0xfffff), // TSS
    ];

    let code_seg = kvm_segment_from_gdt(gdt_table[1], 1);
    let data_seg = kvm_segment_from_gdt(gdt_table[2], 2);
    let tss_seg = kvm_segment_from_gdt(gdt_table[3], 3);

    // Write segments
    write_gdt_table(&gdt_table[..], guest_mem)?;
    sregs.gdt.base = BOOT_GDT_OFFSET;
    sregs.gdt.limit = u16::try_from(std::mem::size_of_val(&gdt_table))? - 1;

    write_idt_value(0, guest_mem)?;
    sregs.idt.base = BOOT_IDT_OFFSET;
    sregs.idt.limit = u16::try_from(std::mem::size_of::<u64>())? - 1;

    sregs.cs = code_seg;
    sregs.ds = data_seg;
    sregs.es = data_seg;
    sregs.fs = data_seg;
    sregs.gs = data_seg;
    sregs.ss = data_seg;
    sregs.tr = tss_seg;

    // 64-bit protected mode
    sregs.cr0 |= X86_CR0_PE;
    sregs.efer |= EFER_LME | EFER_LMA;

    Ok(())
}

fn write_gdt_table(table: &[u64], guest_mem: &GuestMemoryMmap) -> Result<()> {
    let boot_gdt_addr = GuestAddress(BOOT_GDT_OFFSET);

    for (index, entry) in table.iter().enumerate() {
        let addr = guest_mem
            .checked_offset(boot_gdt_addr, index * std::mem::size_of::<u64>())
            .ok_or(anyhow::anyhow!("failed to write GDT"))?;

        guest_mem
            .write_obj(*entry, addr)
            .context("failed to write GDT entry")?;
    }

    Ok(())
}

fn write_idt_value(val: u64, guest_mem: &GuestMemoryMmap) -> Result<()> {
    let boot_idt_addr = GuestAddress(BOOT_IDT_OFFSET);

    guest_mem
        .write_obj(val, boot_idt_addr)
        .context("failed to write IDT address")?;

    Ok(())
}

fn setup_page_tables(guest_mem: &GuestMemoryMmap, sregs: &mut kvm_sregs) -> Result<()> {
    // Puts PML4 right after zero page but aligned to 4k.
    let boot_pml4_addr = GuestAddress(PML4_START);
    let boot_pdpte_addr = GuestAddress(PDPTE_START);
    let boot_pde_addr = GuestAddress(PDE_START);

    // Entry covering VA [0..512GB)
    guest_mem
        .write_obj(boot_pdpte_addr.raw_value() | 0x03, boot_pml4_addr)
        .context("failed to write PML4 address")?;

    // Entry covering VA [0..1GB)
    guest_mem
        .write_obj(boot_pde_addr.raw_value() | 0x03, boot_pdpte_addr)
        .context("failed to write PDPTE address")?;

    // 512 2MB entries together covering VA [0..1GB). Note we are assuming
    // CPU supports 2MB pages (/proc/cpuinfo has 'pse'). All modern CPUs do.
    for i in 0..512 {
        guest_mem
            .write_obj((i << 21) + 0x83u64, boot_pde_addr.unchecked_add(i * 8))
            .context("failed to write PDE address")?;
    }

    sregs.cr3 = boot_pml4_addr.raw_value();
    sregs.cr4 |= X86_CR4_PAE;
    sregs.cr0 |= X86_CR0_PG;

    Ok(())
}
