## Natska HFT Trading Engine

### Inventory benchmark: P99.9 --> 9.99 ns. The new Philosophy: Union-Based Synchronization

A low-latency, deterministic execution engine designed for ultra-high-frequency trading (ULL HFT) environments, achieving a consistent P99.9 latency of 9.99ns on consumer-grade hardware.

![CPU 3GHz Benchmark result](docs/JPEG/Benchmark_P99.9_latency_of_9.99ns_CPU3GHz.png)

![invent_2027_hft_manifesto](docs/JPEG/invent_2027_hft_manifesto.png)

![new_architect_hft_inventory_benchmark](docs/JPEG/new_architect_hft_inventory_benchmark.png)

![memoryOrder__MESI.png](docs/JPEG/memoryOrder__MESI.png.png)

![Lock-free-hot-path-HFT-Low-latency](docs/JPEG/Lock-free-hot-path-HFT-Low-latency.png)

## Core Architecture
    • Lock-Free Design: Implements a high-performance, single-producer, single-consumer (SPSC) ring buffer architecture.

    • Union-Based Synchronization: Utilizes cache-line aware union structures to store head and tail pointers. This technique eliminates "Atomic Battle" by ensuring synchronization updates perform without inducing false sharing or cache-line bouncing (MESI protocol thrashing).

    • Mechanical Sympathy: Optimized for L1/L2 cache locality through strict structural padding (124-byte envelopes), ensuring producer and consumer cores operate on physically distinct cache lines.

## Performance Benchmarks
Metric      Latency
P50         9.99 ns
P99         9.99 ns
P99.9       9.99 ns

## System Tuning & Optimization
## To achieve sub-10ns determinism, the following environment-level optimizations were implemented:
    • Core Isolation: Utilized isolcpus, nohz_full, and rcu_nocbs to reserve dedicated cores for the engine hot-path.
    • Interrupt Affinity: Forced all NIC and device interrupts to Core 0, leaving the Consumer thread on Core 1 completely clear of hardware interrupt jitter.
    • Hardware Hardening: Disabled SMT (Hyper-Threading), C-States, and SpeedStep to prevent CPU power-state transition delays.
    • Residual Latency Analysis: Confirmed that remaining outliers (approx. 9990 cycles) originate from firmware-level System Management Interrupts (SMIs). This represents the theoretical latency floor for the current consumer motherboard platform.
## Future Roadmap
    • Hardware Integration: Upgrading to Solarflare NICs for kernel-bypass networking (OpenOnload/DPDK integration).
    • Network-to-Process Benchmarking: Implementing zero-copy packet parsing from raw sockets to measure end-to-end tick-to-trade latency.


cargo build --release
sudo taskset -c 0,1 ./target/release/natska_engine
(CPU has only 2 physical cores,core0,core1. Core1 is isolated from the kernel but not core0, mother core cannot be isolated from the kernel)

### System Tuning & Kernel Configuration
GRUB_CMDLINE_LINUX_DEFAULT="quiet splash iomem=relaxed intel_iommu=off hugepagesz=2M hugepages=512 isolcpus=1 nohz_full=1 rcu_nocbs=1 irqaffinity=0 intel_pstate=disable kthread_cpus=0"
## Parameter Breakdown
iomem=relaxed: Permits user-space access to physical memory addresses; critical for mapping PCIe BARs for kernel-bypass networking.

intel_iommu=off: Disables the I/O Memory Management Unit to eliminate the latency overhead of DMA remapping, ensuring the most direct path between the NIC and memory.

hugepagesz=2M hugepages=512: Pre-allocates 1GB of contiguous physical memory to eliminate Page Table Walks and TLB (Translation Lookaside Buffer) misses during the hot-path.

isolcpus=1: Removes Core 1 from the Linux scheduler; it will no longer process general-purpose OS tasks, leaving it exclusively for the execution thread.

nohz_full=1: Disables the system tick timer on Core 1, preventing the kernel from interrupting the engine to perform housekeeping tasks.

rcu_nocbs=1: Offloads Read-Copy-Update (RCU) callbacks from the isolated core to prevent unpredictable latency spikes.

irqaffinity=0: Forces all system interrupts (NIC, disk, USB) to be handled by Core 0, ensuring the "clean" core (Core 1) remains uncontended.

intel_pstate=disable: Forces the CPU to run at a fixed frequency, preventing the governor from transitioning between P-states (power states), which is a common source of micro-jitter.

kthread_cpus=0: a boot parameter used to restrict the CPUs on which the kernel can spawn new kernel threads.
When you use this flag:
It forces the kernel to place general-purpose kernel threads and housekeeping tasks exclusively on the specified CPU(s)—in your configuration, Core 0.
This further protects your isolated core (Core 1) by preventing the kernel from "spawning off" background processes or management threads onto the core you have reserved for your low-latency execution engine.
kthread_cpus=0: Restricts the kernel from spawning new kernel threads on any core except Core 0. This acts as a secondary enforcement mechanism to ensure that OS-level housekeeping tasks never migrate onto the isolated "hot-path" core.




![ Inventory benchmark architect 2026: Union-Based Synchronization](docs/JPEG/Inventory_benchmark_architect_2026_Union_Based_Synchronization_Tail_Head.png)

![driverless](docs/JPEG/driverless.png)

![ Inventory benchmark architect 2026: Union-Based Synchronization Head & Heal](docs/JPEG/inventory_benchmark_architect_2026_Union_Based_Synchronization.png)


![No RTOS, NO Kernel Preemt, No kernel noise. Loops sync done by sfence & lfence](docs/JPEG/Producer_Consumer_Flow_Control_Diagram.png)


![Natska-Rule++ Architecture](docs/JPEG/Natska_Rule_Architecture.png)

![Old bad result before the kthread_cpus=0 & BIOS thread dependency settings ](docs/JPEG/old_benchmark_13ns__P99.9929CPU3GHz.png)

![CPU_Synchronization_Hardware Sync](docs/JPEG/CPU_Synchronization_Hardware.png)

![The hardware_sync. Indirectly read / write. Let it know what you want as publishing. It will do it safely without race condition. Thanks to it, the consumer & producer loops made without kernel preempt, rtos, timer event, any rt scheduling, any noise at all. ](docs/JPEG/hardware_sync.png)

![](docs/JPEG/park_warehouse.png)

![bare_metal_network_setup_Driverless_No_kernel](docs/JPEG/bare_metal_network_setup_Driverless_No_kernel.png)

![check_kernel_security](docs/JPEG/check_kernel_security.png)

![DL_ML_model_alpha_exe_infra_structure](docs/JPEG/DL_ML_model_alpha_exe_infra_structure.png)

![zero_kernel_noise](docs/JPEG/zero_kernel_noise.png)

![zero_hardware_drop](docs/JPEG/zero_hardware_drop.png)

![who_does_what](docs/JPEG/who_does_what.png)

![Slab_failure](docs/JPEG/Slab_failure.png)

![NIC_accesses](docs/JPEG/NIC_accesses.png)

![Linker's paradox](docs/JPEG/Linker's_paradox.png)

![L1L2_caches. My latest benchmark ison the L1. Because it has 10ns. It means there is no chance to be in the L2! We are the winner!](docs/JPEG/L1L2_caches.png)

![huge_page_1GB_map](docs/JPEG/huge_page_1GB_map.png)

![Hot_path](docs/JPEG/Hot_path.png)

![HFT-Low_latency](docs/JPEG/HFT-Low_latency.png)

![hardware_sync_MESI_fences](docs/JPEG/hardware_sync_MESI_fences.png)

![hardware_BAR_map](docs/JPEG/hardware_BAR_map.png)

![GRUB_settings](docs/JPEG/GRUB_settings.png)

![disable_kernel_noises](docs/JPEG/hardware/disable_kernel_noises.png)

![driverless_control_DMA](docs/JPEG/driverless_control_DMA.png)

![driverless_control_SegFault_but_via_mmap](docs/JPEG/driverless_control_SegFault_but_via_mmap.png)


![driverless_control_via_mmap](docs/JPEG/driverless_control_via_mmap.png)

![driverless_control_via_mmap](docs/JPEG/driverless_control_via_mmap.png)


![driverless_control_zero_jitter](docs/JPEG/driverless_control_zero_jitter.png)

![driverless_NIC_control_by_user](docs/JPEG/driverless_NIC_control_by_user.png)


![driverless_NIC_control_DMA_Slab](docs/JPEG/driverless_NIC_control_DMA_Slab.png)

![driverless_NIC_control_metaphor](docs/JPEG/driverless_NIC_control_metaphor.png)

![driverless_NIC_control_room](docs/JPEG/driverless_NIC_control_room.png)

![driverless_NIC_control](docs/JPEG/driverless_NIC_control.png)

![driverless_NIC_Physical_address](docs/JPEG/driverless_NIC_Physical_address.png)

![dpdk_inside_NIC](docs/JPEG/dpdk_inside_NIC.png)

![priroity](docs/JPEG/HFT_banking_architect/priroity.png)

![Solana_turbine_propagation](docs/JPEG/HFT_banking_architect/Solana_turbine_propagation.png)

![Tx_Rx_ring_buffers](docs/JPEG/hardware/Tx_Rx_ring_buffers.png)

![rte_mbuf_structure](docs/JPEG/hardware/rte_mbuf_structure.png)

![race_condition_sorted](docs/JPEG/hardware/race_condition_sorted.png)

![panic_double_loading](docs/JPEG/hardware/panic_double_loading.png)

![No_iommu](docs/JPEG/hardware/No_iommu.png)

![no_dynamic_but_static_linkage](docs/JPEG/hardware/no_dynamic_but_static_linkage.png)

![NIC_in_jail](docs/JPEG/hardware/NIC_in_jail.png)

![NIC_in_jail_12](docs/JPEG/hardware/NIC_in_jail_12.png)

![Mbuffer_pool](docs/JPEG/hardware/Mbuffer_pool.png)

![kernel_is_an_enemy](docs/JPEG/hardware/kernel_is_an_enemy.png)

![DPDK_has_NIC_we_have_buffers](docs/JPEG/hardware/DPDK_has_NIC_we_have_buffers.png)

![DPDK_BUT_No_kernel_stack_No_copy](docs/JPEG/hardware/DPDK_BUT_No_kernel_stack_No_copy.png)

![core_local_mem](docs/JPEG/hardware/core_local_mem.png)

![aggregator](docs/JPEG/hardware/DPDK/aggregator.png)

![corrupt_DPDK](docs/JPEG/hardware/DPDK/corrupt_DPDK.png)


![DPDK_kernel_bypass](docs/JPEG/hardware/DPDK/DPDK_kernel_bypass.png)

![dpdk_zero_kernel_noise](docs/JPEG/hardware/DPDK/dpdk_zero_kernel_noise.png)


![manage_kernel_bugs](docs/JPEG/hardware/DPDK/manage_kernel_bugs.png)

![No_sys_libs_but_custom_DPDK](docs/JPEG/hardware/DPDK/No_sys_libs_but_custom_DPDK.png)


![plug_unplug_NIC](docs/JPEG/hardware/DPDK/plug_unplug_NIC.png)

![Skeleton_nic_rtl8168_native_PMD](docs/JPEG/hardware/DPDK/Skeleton_nic_rtl8168_native_PMD.png)


![Solana_MTU](docs/JPEG/hardware/DPDK/Solana_MTU.png)

