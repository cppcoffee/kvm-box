use anyhow::{Context, Result};
use kvm_bindings::{kvm_userspace_memory_region, KVM_MEM_LOG_DIRTY_PAGES};
use kvm_ioctls::VmFd;
use vm_memory::{GuestAddress, GuestMemory, GuestMemoryMmap};

const RAM_BASE: u64 = 0;

pub fn create_guest_memory(vm: &VmFd, ram_size: u64) -> Result<GuestMemoryMmap> {
    let guest_addr = GuestAddress(RAM_BASE);

    let guest_mem = GuestMemoryMmap::<()>::from_ranges(&[(guest_addr, ram_size as usize)])
        .context("failed to create guest memory")?;

    let host_addr = guest_mem
        .get_host_address(guest_addr)
        .context("failed to get host address")?;

    let mem_region = kvm_userspace_memory_region {
        slot: 0,
        guest_phys_addr: RAM_BASE,
        memory_size: ram_size,
        userspace_addr: host_addr as u64,
        flags: KVM_MEM_LOG_DIRTY_PAGES,
    };

    unsafe {
        vm.set_user_memory_region(mem_region)
            .context("failed set user memory region")?;
    }

    vm.set_tss_address(crate::arch::layout::KVM_TSS_ADDRESS)
        .context("failed to set tss address")?;

    Ok(guest_mem)
}
