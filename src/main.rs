use std::path::PathBuf;

use anyhow::{Context, Result};
use log::error;
use vmm_sys_util::terminal::Terminal;

mod arch;
mod devices;
mod vmm;
use vmm::Vmm;

#[derive(argh::FromArgs, Debug)]
#[argh(description = "A simple hypervisor")]
struct Args {
    #[argh(option, long = "kernel", description = "path to the kernel image")]
    kernel: Option<PathBuf>,

    #[argh(option, long = "cmdline", description = "kernel boot cmdline")]
    boot_cmdline: Option<String>,

    #[argh(option, long = "initrd", description = "path to the initrd")]
    initrd: Option<PathBuf>,

    #[argh(
        switch,
        short = 'v',
        long = "version",
        description = "print version info"
    )]
    version: bool,
}

fn main() -> Result<()> {
    // We need this so that we can reset terminal to canonical mode if panic occurs.
    let stdin = std::io::stdin();

    std::panic::set_hook(Box::new(move |info| {
        error!("kvm-box {}", info);

        if let Err(err) = stdin.lock().set_canon_mode() {
            error!(
                "Failure while trying to reset stdin to canonical mode: {}",
                err
            );
        }
    }));

    let args = argh::from_env::<Args>();

    if args.version {
        print_version();
        return Ok(());
    }

    let kernel = args
        .kernel
        .ok_or(anyhow::anyhow!("kernel argument required"))?;

    let ram_size = 0x8000_0000; // 2G
    let mut vm = Vmm::new(ram_size).context("failed to create vmm")?;
    vm.init().context("failed to vmm.init")?;

    let boot_source_cfg = arch::BootSourceConfig {
        kernel_image_path: kernel.to_string_lossy().to_string(),
        initrd_path: args.initrd.map(|p| p.to_string_lossy().to_string()),
        boot_args: args.boot_cmdline,
    };

    vm.load_image(&boot_source_cfg)
        .context("failed to load image")?;

    vm.run().context("failed to vmm.run")?;

    std::io::stdin()
        .lock()
        .set_canon_mode()
        .context("failed to reset stdin to canonical mode")?;

    Ok(())
}

fn print_version() {
    println!(
        "{} {}",
        std::env!("CARGO_PKG_NAME"),
        std::env!("CARGO_PKG_VERSION")
    );

    println!("{}\n", std::env!("CARGO_PKG_DESCRIPTION"));
    println!("Written by {}", std::env!("CARGO_PKG_AUTHORS"));
}
