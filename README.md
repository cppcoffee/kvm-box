# kvm-box

The `kvm-box` is a minimalist virtual machine monitor (VMM) that uses the Linux Kernel Virtual Machine (KVM) to create and run microVM, capable of running Linux kernel partially.

## Supported Architecture

- x86-64

## Build

```shell
$ make
```

The `kvm-box` binary will be placed at `target/release/kvm-box`.

## Preparing the kernel and initrd

kernel: use pre-compiled and tuned files from firecracker:

```shell
$ wget https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/kernels/vmlinux.bin
```

initrd:

```shell
$ git clone https://github.com/marcov/firecracker-initrd.git
$ cd firecracker-initrd
$ bash -x ./build.sh
```

## Usage

```shell
$ ./target/release/kvm-box --kernel ./testdata/vmlinux.bin --initrd ./testdata/initrd.img
[    0.000000] Linux version 4.14.174 (@57edebb99db7) (gcc version 7.5.0 (Ubuntu 7.5.0-3ubuntu1~18.04)) #2 SMP Wed Jul 14 11:47:24 UTC 2021
[    0.000000] Command line: console=ttyS0 noapic noacpi reboot=k panic=1 pci=off nomodule
[    0.000000] Disabled fast string operations
[    0.000000] x86/fpu: Supporting XSAVE feature 0x001: 'x87 floating point registers'
[    0.000000] x86/fpu: Supporting XSAVE feature 0x002: 'SSE registers'
[    0.000000] x86/fpu: Supporting XSAVE feature 0x004: 'AVX registers'
[    0.000000] x86/fpu: xstate_offset[2]:  576, xstate_sizes[2]:  256
[    0.000000] x86/fpu: Enabled xstate features 0x7, context size is 832 bytes, using 'standard' format.
[    0.000000] e820: BIOS-provided physical RAM map:
[    0.000000] BIOS-e820: [mem 0x0000000000000000-0x000000000009fbff] usable
[    0.000000] BIOS-e820: [mem 0x000000000009fc00-0x000000000009ffff] reserved
[    0.000000] BIOS-e820: [mem 0x0000000000100000-0x000000007fffffff] usable
[    0.000000] NX (Execute Disable) protection: active
[    0.000000] DMI not present or invalid.
[    0.000000] tsc: Fast TSC calibration failed
[    0.000000] tsc: Unable to calibrate against PIT
[    0.000000] tsc: No reference (HPET/PMTIMER) available
[    0.000000] e820: last_pfn = 0x80000 max_arch_pfn = 0x400000000
[    0.000000] MTRR: Disabled
[    0.000000] x86/PAT: MTRRs disabled, skipping PAT initialization too.
[    0.000000] CPU MTRRs all blank - virtualized system.
[    0.000000] x86/PAT: Configuration [0-7]: WB  WT  UC- UC  WB  WT  UC- UC
[    0.000000] Scanning 1 areas for low memory corruption
[    0.000000] Using GB pages for direct mapping
[    0.000000] RAMDISK: [mem 0x7f180000-0x7fffffff]
[    0.000000] No NUMA configuration found
[    0.000000] Faking a node at [mem 0x0000000000000000-0x000000007fffffff]
[    0.000000] NODE_DATA(0) allocated [mem 0x7f15e000-0x7f17ffff]
[    0.000000] Zone ranges:
[    0.000000]   DMA      [mem 0x0000000000001000-0x0000000000ffffff]
[    0.000000]   DMA32    [mem 0x0000000001000000-0x000000007fffffff]
[    0.000000]   Normal   empty
[    0.000000] Movable zone start for each node
[    0.000000] Early memory node ranges
[    0.000000]   node   0: [mem 0x0000000000001000-0x000000000009efff]
[    0.000000]   node   0: [mem 0x0000000000100000-0x000000007fffffff]
[    0.000000] Initmem setup node 0 [mem 0x0000000000001000-0x000000007fffffff]
[    0.000000] smpboot: Boot CPU (id 0) not listed by BIOS
[    0.000000] smpboot: Allowing 1 CPUs, 0 hotplug CPUs
[    0.000000] PM: Registered nosave memory: [mem 0x00000000-0x00000fff]
[    0.000000] PM: Registered nosave memory: [mem 0x0009f000-0x0009ffff]
[    0.000000] PM: Registered nosave memory: [mem 0x000a0000-0x000fffff]
[    0.000000] e820: [mem 0x80000000-0xffffffff] available for PCI devices
[    0.000000] Booting paravirtualized kernel on bare hardware
[    0.000000] clocksource: refined-jiffies: mask: 0xffffffff max_cycles: 0xffffffff, max_idle_ns: 7645519600211568 ns
[    0.000000] random: get_random_bytes called from start_kernel+0x94/0x486 with crng_init=0
[    0.000000] setup_percpu: NR_CPUS:128 nr_cpumask_bits:128 nr_cpu_ids:1 nr_node_ids:1
[    0.000000] percpu: Embedded 41 pages/cpu s128600 r8192 d31144 u2097152
[    0.000000] Built 1 zonelists, mobility grouping on.  Total pages: 515977
[    0.000000] Policy zone: DMA32
[    0.000000] Kernel command line: console=ttyS0 noapic noacpi reboot=k panic=1 pci=off nomodule
[    0.000000] PID hash table entries: 4096 (order: 3, 32768 bytes)
[    0.000000] Memory: 2031508K/2096760K available (8204K kernel code, 645K rwdata, 1480K rodata, 1324K init, 2792K bss, 65252K reserved, 0K cma-reserved)
[    0.000000] SLUB: HWalign=64, Order=0-3, MinObjects=0, CPUs=1, Nodes=1
[    0.000000] Kernel/User page tables isolation: enabled
[    0.000000] Hierarchical RCU implementation.
[    0.000000]  RCU restricting CPUs from NR_CPUS=128 to nr_cpu_ids=1.
[    0.000000] RCU: Adjusting geometry for rcu_fanout_leaf=16, nr_cpu_ids=1
[    0.000000] NR_IRQS: 4352, nr_irqs: 24, preallocated irqs: 16
[    0.000000] Console: colour dummy device 80x25
[    0.000000] console [ttyS0] enabled
[    0.000000] tsc: Fast TSC calibration failed
[    0.016000] tsc: Unable to calibrate against PIT
[    0.020000] tsc: No reference (HPET/PMTIMER) available
[    0.028000] tsc: Marking TSC unstable due to could not calculate TSC khz
[    0.036000] Calibrating delay loop... 5607.42 BogoMIPS (lpj=11214848)
[    0.064000] pid_max: default: 32768 minimum: 301
[    0.072000] Security Framework initialized
[    0.076000] SELinux:  Initializing.
[    0.124000] Dentry cache hash table entries: 262144 (order: 9, 2097152 bytes)
[    0.148000] Inode-cache hash table entries: 131072 (order: 8, 1048576 bytes)
[    0.156000] Mount-cache hash table entries: 4096 (order: 3, 32768 bytes)
[    0.164000] Mountpoint-cache hash table entries: 4096 (order: 3, 32768 bytes)
[    0.176000] Disabled fast string operations
[    0.184000] Last level iTLB entries: 4KB 0, 2MB 0, 4MB 0
[    0.188000] Last level dTLB entries: 4KB 0, 2MB 0, 4MB 0, 1GB 0
[    0.196000] Spectre V1 : Mitigation: usercopy/swapgs barriers and __user pointer sanitization
[    0.204000] Spectre V2 : Mitigation: Full generic retpoline
[    0.212000] Spectre V2 : Spectre v2 / SpectreRSB mitigation: Filling RSB on context switch
[    0.216000] Spectre V2 : Enabling Restricted Speculation for firmware calls
[    0.224000] Spectre V2 : mitigation: Enabling conditional Indirect Branch Prediction Barrier
[    0.228000] Speculative Store Bypass: Mitigation: Speculative Store Bypass disabled via prctl and seccomp
[    0.236000] MDS: Mitigation: Clear CPU buffers
[    0.308000] Freeing SMP alternatives memory: 28K
[    0.332000] smpboot: Max logical packages: 1
[    0.336000] smpboot: CPU 0 Converting physical 29 to logical package 0
[    0.340000] smpboot: SMP motherboard not detected
[    0.344000] smpboot: SMP disabled
[    0.348000] Not enabling interrupt remapping due to skipped IO-APIC setup
[    0.576000] Performance Events: unsupported p6 CPU model 60 no PMU driver, software events only.
[    0.580000] Hierarchical SRCU implementation.
[    0.588000] smp: Bringing up secondary CPUs ...
[    0.592000] smp: Brought up 1 node, 1 CPU
[    0.596000] smpboot: Total of 1 processors activated (5607.42 BogoMIPS)
[    0.600000] devtmpfs: initialized
[    0.608000] x86/mm: Memory block size: 128MB
[    0.616000] clocksource: jiffies: mask: 0xffffffff max_cycles: 0xffffffff, max_idle_ns: 7645041785100000 ns
[    0.620000] futex hash table entries: 256 (order: 2, 16384 bytes)
[    0.632000] NET: Registered protocol family 16
[    0.636000] cpuidle: using governor ladder
[    0.640000] cpuidle: using governor menu
[    0.688000] HugeTLB registered 1.00 GiB page size, pre-allocated 0 pages
[    0.692000] HugeTLB registered 2.00 MiB page size, pre-allocated 0 pages
[    0.696000] SCSI subsystem initialized
[    0.700000] pps_core: LinuxPPS API ver. 1 registered
[    0.704000] pps_core: Software ver. 5.3.6 - Copyright 2005-2007 Rodolfo Giometti <giometti@linux.it>
[    0.708000] PTP clock support registered
[    0.712000] dmi: Firmware registration failed.
[    0.716000] NetLabel: Initializing
[    0.720000] NetLabel:  domain hash size = 128
[    0.724000] NetLabel:  protocols = UNLABELED CIPSOv4 CALIPSO
[    0.728000] NetLabel:  unlabeled traffic allowed by default
[    0.732000] clocksource: Switched to clocksource refined-jiffies
[    0.736000] VFS: Disk quotas dquot_6.6.0
[    0.740000] VFS: Dquot-cache hash table entries: 512 (order 0, 4096 bytes)
[    0.752001] NET: Registered protocol family 2
[    0.756001] TCP established hash table entries: 16384 (order: 5, 131072 bytes)
[    0.760001] TCP bind hash table entries: 16384 (order: 6, 262144 bytes)
[    0.764002] TCP: Hash tables configured (established 16384 bind 16384)
[    0.768002] UDP hash table entries: 1024 (order: 3, 32768 bytes)
[    0.772002] UDP-Lite hash table entries: 1024 (order: 3, 32768 bytes)
[    0.776002] NET: Registered protocol family 1
[    0.780003] Unpacking initramfs...
[    1.240031] Freeing initrd memory: 14848K
[    1.244032] platform rtc_cmos: registered platform RTC device (no PNP device found)
[    1.248032] Scanning for low memory corruption every 60 seconds
[    1.252032] audit: initializing netlink subsys (disabled)
[    1.256032] Initialise system trusted keyrings
[    1.260033] Key type blacklist registered
[    1.264033] audit: type=2000 audit(943920001.256:1): state=initialized audit_enabled=0 res=1
[    1.268033] workingset: timestamp_bits=36 max_order=19 bucket_order=0
[    1.292035] squashfs: version 4.0 (2009/01/31) Phillip Lougher
[    1.300035] Key type asymmetric registered
[    1.304035] Asymmetric key parser 'x509' registered
[    1.308036] Block layer SCSI generic (bsg) driver version 0.4 loaded (major 252)
[    1.312036] io scheduler noop registered (default)
[    1.316036] io scheduler cfq registered
[    1.320036] Serial: 8250/16550 driver, 1 ports, IRQ sharing disabled
[    1.328037] serial8250: ttyS0 at I/O 0x3f8 (irq = 4, base_baud = 115200) is a U6_16550A
[    1.336037] loop: module loaded
[    1.340038] Loading iSCSI transport class v2.0-870.
[    1.344038] iscsi: registered transport (tcp)
[    1.348038] tun: Universal TUN/TAP device driver, 1.6
[    1.356039] i8042: Can't read CTR while initializing i8042
[    1.360039] i8042: probe of i8042 failed with error -5
[    1.364039] hidraw: raw HID events driver (C) Jiri Kosina
[    1.368039] nf_conntrack version 0.5.0 (16384 buckets, 65536 max)
[    1.372040] ip_tables: (C) 2000-2006 Netfilter Core Team
[    1.376040] Initializing XFRM netlink socket
[    1.384040] NET: Registered protocol family 10
[    1.392041] Segment Routing with IPv6
[    1.396041] NET: Registered protocol family 17
[    1.400041] Bridge firewalling registered
[    1.404042] NET: Registered protocol family 40
[    1.408042] registered taskstats version 1
[    1.412042] Loading compiled-in X.509 certificates
[    1.416042] Loaded X.509 cert 'Build time autogenerated kernel key: e98e9d271da5d0a322cc4d7bfaa8c2c4c3e46010'
[    1.424043] Key type encrypted registered
[    1.440044] Freeing unused kernel memory: 1324K
[    1.452045] Write protecting the kernel read-only data: 12288k
[    1.488047] Freeing unused kernel memory: 2016K
[    1.500048] Freeing unused kernel memory: 568K

   OpenRC 0.52.1 is starting up Linux 4.14.174 (x86_64)

 * Mounting /proc ... [ ok ]
 * Mounting /run ... [ ok ]
 * /run/openrc: creating directory
 * /run/lock: creating directory
 * /run/lock: correcting owner
 * Caching service dependencies ... [ ok ]
 * Clock skew detected with `/etc/init.d'
 * Adjusting mtime of `/run/openrc/deptree' to Tue Apr 30 14:22:13 2024

 * WARNING: clock skew detected!
 * WARNING: clock skew detected!
 * Mounting devtmpfs on /dev ... [ ok ]
 * Mounting /dev/mqueue ... [ ok ]
 * Mounting /dev/pts ... [ ok ]
 * Mounting /dev/shm ... [ ok ]
 * Loading modules ...modprobe: can't change directory to '/lib/modules': No such file or directory
modprobe: can't change directory to '/lib/modules': No such file or directory
 [ ok ]
 * Mounting misc binary format filesystem ... [ ok ]
 * Mounting /sys ... [ ok ]
 * Mounting security filesystem ... [ ok ]
 * Mounting debug filesystem ... [ ok ]
 * Mounting SELinux filesystem ... [ ok ]
 * Mounting persistent storage (pstore) filesystem ... [ ok ]
 * WARNING: clock skew detected!

Welcome to Alpine Linux 3.19
Kernel 4.14.174 on an x86_64 (ttyS0)

(none) login: root
Welcome to Alpine!
[    4.208217] random: fast init done

The Alpine Wiki contains a large amount of how-to guides and general
information about administrating Alpine systems.
See <https://wiki.alpinelinux.org/>.

You can setup the system with the command: setup-alpine

You may change this message by editing /etc/motd.

login[818]: root login on 'ttyS0'
(none):~# uname -a
Linux (none) 4.14.174 #2 SMP Wed Jul 14 11:47:24 UTC 2021 x86_64 Linux
(none):~# reboot -f
[    8.888509] reboot: Restarting system
[    8.892510] reboot: machine restart
KVM_EXIT_SHUTDOWN
vcpu stopped, main loop exit
```

## References

[KVM (Kernel-based Virtual Machine) API](https://www.kernel.org/doc/Documentation/virtual/kvm/api.txt)

[Using the KVM API](https://lwn.net/Articles/658511/)

[firecracker](https://github.com/firecracker-microvm/firecracker)

[firecracker-initrd](https://github.com/marcov/firecracker-initrd)

