use anyhow::{Context, Result};
use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
use log::{error, info};
use vm_memory::GuestMemoryMmap;
use vm_superio::Trigger;
use vmm_sys_util::{poll::PollContext, terminal::Terminal};

use crate::devices::{setup_serial_device, Bus, EventFdTrigger, PortIODeviceManager};

pub struct Vmm {
    pub kvm: Kvm,
    pub vm: VmFd,
    pub guest_mem: GuestMemoryMmap,
    pub vcpu: Option<VcpuFd>,
    pub pio_device_manager: Option<PortIODeviceManager>,
}

impl Vmm {
    pub fn new(ram_size: u64) -> Result<Vmm> {
        let kvm = Kvm::new().context("failed to create kvm")?;
        let vm = kvm.create_vm().context("failed to create vm")?;

        crate::arch::irq::init_irqchip(&vm).context("failed to init irq chip")?;

        let guest_mem = crate::arch::memory::create_guest_memory(&vm, ram_size)?;

        Ok(Vmm {
            kvm,
            vm,
            guest_mem,
            vcpu: None,
            pio_device_manager: None,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        let vcpu = self.vm.create_vcpu(0).context("failed to create vcpu")?;

        crate::arch::vcpu::init_cpu_id(&self.kvm, &vcpu)?;

        // TODO: init msrs

        crate::arch::regs::init_regs(&vcpu, crate::arch::layout::KERNEL_START_ADDRESS)?;
        crate::arch::regs::init_fpu(&vcpu)?;
        crate::arch::regs::init_sregs(&self.guest_mem, &vcpu)?;

        self.vcpu = Some(vcpu);

        Ok(())
    }

    pub fn load_image(&self, boot_source_cfg: &crate::arch::BootSourceConfig) -> Result<()> {
        crate::arch::system::load_kernel(&boot_source_cfg.kernel_image_path, &self.guest_mem)
            .context("failed to load kernel")?;

        let initrd = match &boot_source_cfg.initrd_path {
            Some(p) => Some(
                crate::arch::system::load_initrd(p, &self.guest_mem)
                    .context("failed to load initrd")?,
            ),
            None => None,
        };

        let (cmdline_addr, cmdline_size) =
            crate::arch::system::load_boot_cmdline(&self.guest_mem, &boot_source_cfg)
                .context("failed to load boot cmdline")?;

        crate::arch::system::configure_system(
            &self.guest_mem,
            cmdline_addr,
            cmdline_size,
            &initrd,
        )?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        let serial_device = setup_serial_device(std::io::stdin(), std::io::stdout())?;
        let mut pio_device_manager = PortIODeviceManager::new(serial_device.clone())?;
        pio_device_manager.register_devices(&self.vm)?;

        let vcpu_exit_evt = self.start_threaded(pio_device_manager.io_bus.clone())?;

        let stdin = std::io::stdin().lock();
        stdin
            .set_raw_mode()
            .context("failed to set terminal raw mode")?;
        stdin
            .set_non_block(true)
            .context("failed to set terminal non block mode")?;

        let poll_ctx: PollContext<u8> =
            PollContext::new().context("failed to create epoll context")?;

        poll_ctx.add(&vcpu_exit_evt.0, 0)?;
        poll_ctx.add(&stdin, 1)?;

        self.pio_device_manager = Some(pio_device_manager);

        loop {
            let events = poll_ctx.wait().context("failed to wait for events")?;
            for ev in events.iter_readable() {
                match ev.token() {
                    0 => {
                        info!("vcpu stopped, main loop exit");
                        return Ok(());
                    }
                    1 => {
                        let mut out = [0u8; 64];
                        match stdin.read_raw(&mut out[..]) {
                            Ok(0) => {}
                            Ok(n) => {
                                serial_device
                                    .lock()
                                    .expect("Poisoned lock")
                                    .serial_mut()
                                    .unwrap()
                                    .serial
                                    .enqueue_raw_bytes(&out[..n])
                                    .expect("enqueue bytes failed");
                            }
                            Err(e) => {
                                error!("error while reading stdin: {:?}", e);
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn start_threaded(&mut self, pio_bus: Bus) -> Result<EventFdTrigger> {
        let vcpu = match std::mem::take(&mut self.vcpu) {
            Some(vcpu) => vcpu,
            None => return Err(anyhow::anyhow!("vcpu is not initialized")),
        };

        let exit_evt = EventFdTrigger::new();
        let vcpu_exit_evt = exit_evt.try_clone().context("failed to clone eventfd")?;

        let builder = std::thread::Builder::new();
        let _ = builder
            .name(String::from("vcpu0"))
            .spawn(move || {
                loop {
                    match vcpu.run() {
                        Ok(run) => match run {
                            VcpuExit::IoIn(addr, data) => {
                                pio_bus.read(addr.into(), data);
                            }
                            VcpuExit::IoOut(addr, data) => {
                                pio_bus.write(addr.into(), data);
                            }
                            VcpuExit::MmioRead(_, _) => {
                                info!("mmio read");
                            }
                            VcpuExit::MmioWrite(_, _) => {
                                info!("mmio write");
                            }
                            VcpuExit::Hlt => {
                                info!("KVM_EXIT_HLT");
                                break;
                            }
                            VcpuExit::Shutdown => {
                                info!("KVM_EXIT_SHUTDOWN");
                                break;
                            }
                            r => {
                                info!("KVM_EXIT: {:?}", r);
                                break;
                            }
                        },

                        Err(e) => {
                            error!("vm run error: {:?}", e);
                            break;
                        }
                    }
                }

                exit_evt.trigger().expect("failed to write to exit_evt");
            })
            .context("failed to spawn vcpu thread");

        Ok(vcpu_exit_evt)
    }
}
