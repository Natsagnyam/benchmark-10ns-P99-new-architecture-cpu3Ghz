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
sudo ./target/release/natska_engine

```



## Project Structure

* `src/lib.rs`: Core structures, memory barriers, and hardware primitives.
* `src/main.rs`: High-frequency hot-path loop and benchmark collection.
* `docs/`: Detailed performance specifications and architectural diagrams.

---

*Developed under the Natska_Rule++ performance standard.*

How would you like to proceed with the documentation—would you like me to add a section on how to interpret the benchmark histograms in this README?