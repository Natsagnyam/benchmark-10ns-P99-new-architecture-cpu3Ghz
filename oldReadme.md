# Natska_Rule++ HFT Engine

## Overview

**Natska_Rule++** is a high-frequency trading (HFT) engine designed for deterministic, sub-10ns execution on commodity x86_64 hardware. By bypassing standard OS abstractions and pinning execution to isolated CPU cores, the engine achieves near-hardware-level performance with minimal jitter.

## Core Architecture

The engine utilizes a zero-allocation, lock-free ring buffer architecture, ensuring that the critical path remains within the CPU's L1 cache.

* **Cache-Line Isolation**: Structures (`ProducerCtrl`, `ConsumerCtrl`) are padded to 124 bytes plus the variable, ensuring they occupy distinct cache lines to suppress MESI protocol snooping traffic.
* **Hardware Pinning**: Utilizes `isolcpus` and `nohz_full` kernel boot parameters to dedicate cores entirely to the execution loop.
* **Deterministic Timing**: Direct emission of `rdtscp` instructions allows for high-precision latency measurement without the overhead of system-call-based timers.
* **Zero-Copy Logic**: Data moves directly from producer to consumer memory without intermediate buffers or heap allocations.

## Performance Metrics

The system is optimized for the "Golden Path" of operation, with performance profiles validated via cycle-accurate histograms.

| Metric | Latency |
| --- | --- |
| **Median (P50)** | ~10 ns (30 cycles) |
| **P99.9** | ~13 ns (40 cycles) |
| **Tail (P99.99+)** | <20 ns (70 cycles) |

*System calibrated on Intel Haswell architecture (ASROCK H81M) @ 3.0+ GHz.*

## Build & Deployment

The build process enforces `panic = "abort"` and `opt-level = 3` to eliminate software-induced jitter.

1. **System Preparation**: Ensure kernel isolation (`isolcpus`, `nohz_full`, `rcu_nocbs`) is configured in GRUB.
2. **Compilation**:
```bash
cargo build --release

```


3. **Execution**: Must be run with `sudo` to permit `mlockall` and thread affinity operations.
```bash

##  Pinning to core 1 and 2
sudo taskset -c 1,2 ./target/release/natska_engine
# Pin the engine to both cores, allowing the thread-internal affinity to take over
sudo taskset -c 0,1 ./target/release/natska_engine

```



## Project Structure

* `src/lib.rs`: Core structures, memory barriers, and hardware primitives.
* `src/main.rs`: High-frequency hot-path loop and benchmark collection.
* `docs/`: Detailed performance specifications and architectural diagrams.

---

*Developed under the Natska_Rule++ performance standard.*

![CPU 3GHz Benchmark result:](docs/JPEG/benchmark_13ns__P99.9929CPU3GHz.png)
![CPU 3GHz Benchmark result:](docs/JPEG/top.png)


#GRUB_CMDLINE_LINUX_DEFAULT="quiet splash iomem=relaxed intel_iommu=off hugepagesz=2M hugepages=512 isolcpus=1,2 nohz_full=1,2 rcu_nocbs=1,2 irqaffinity=1 intel_pstate=disable process>
GRUB_CMDLINE_LINUX_DEFAULT="quiet splash iomem=relaxed intel_iommu=off hugepagesz=2M hugepages=512 isolcpus=1,2,3,4 nohz_full=1,2,3,4 rcu_nocbs=1,2,3,4 irqaffinity=0 intel_pstate=disa>


GRUB_CMDLINE_LINUX=""



#GRUB_CMDLINE_LINUX_DEFAULT="quiet splash iomem=relaxed intel_iommu=off hugepagesz=2M hugepages=512 isolcpus=1,2 nohz_full=1,2 rcu_nocbs=1,2 irqaffinity=1 intel_pstate=disable process>
GRUB_CMDLINE_LINUX_DEFAULT="quiet splash iomem=relaxed intel_iommu=off hugepagesz=2M hugepages=512 isolcpus=1,2,3,4 nohz_full=1,2,3,4 rcu_nocbs=1,2,3,4 irqaffinity=0 intel_pstate=disa>
GRUB_CMDLINE_LINUX_DEFAULT="quiet splash iomem=relaxed intel_iommu=off hugepagesz=2M hugepages=512 isolcpus=1 nohz_full=1 rcu_nocbs=1 irqaffinity=0 intel_pstate=disable kthread_cpus=0"

GRUB_CMDLINE_LINUX_DEFAULT="... isolcpus=1 nohz_full=1 rcu_nocbs=1 irqaffinity=0 kthread_cpus=0"

