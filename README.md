## Natska HFT Trading Engine

### Inventory benchmark: P99.9 --> 9.99 ns. The new Philosophy: Union-Based Synchronization

A low-latency, deterministic execution engine designed for ultra-high-frequency trading (ULL HFT) environments, achieving a consistent P99.9 latency of 9.99ns on consumer-grade hardware.

![CPU 3GHz Benchmark result](docs/JPEG/Benchmark P99.9 latency of 9.99ns_CPU3GHz.png)

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




![ Inventory benchmark architect 2026: Union-Based Synchronization](docs/JPEG/inventory benchmark architect 2026: Union-Based Synchronization.png)
![ Inventory benchmark architect 2026: Union-Based Synchronization Head & Heal](docs/JPEG/Inventory benchmark architect 2026: Union-Based Synchronization Tail Head.png)


![No RTOS, NO Kernel Preemt, No kernel noise. Loops sync done by sfence & lfence](docs/JPEG/Producer & Consumer Flow Control Diagram.png)