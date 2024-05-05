use anyhow::{Context, Result};
use kvm_bindings::KVM_MAX_CPUID_ENTRIES;
use kvm_ioctls::{Kvm, VcpuFd};

const KVM_CPUID_SIGNATURE: u32 = 0x40000000;
const KVM_CPUID_FEATURES: u32 = 0x40000001;

// KVM CPU feature flags
pub fn init_cpu_id(vm: &Kvm, vcpu: &VcpuFd) -> Result<()> {
    let mut cpuid = vm
        .get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)
        .context("failed to get supported cpuid")?;

    let entries = cpuid.as_mut_slice();

    for entry in entries.iter_mut() {
        if entry.function == KVM_CPUID_SIGNATURE {
            entry.eax = KVM_CPUID_FEATURES;
            entry.ebx = 0x4b4d564b; // KVMK
            entry.ecx = 0x564b4d56; // VMKV
            entry.edx = 0x4d; // M
        }
    }

    vcpu.set_cpuid2(&cpuid).context("failed to set cpuid2")?;

    Ok(())
}
