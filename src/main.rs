// src/main.rs
mod lib; // This tells the compiler to include your structures/primitives

use crate::lib::{ProducerCtrl, ConsumerCtrl, Index, LATENCY_HISTOGRAM, read_tsc, print_latency_report,
    publish_barrier, acquire_barrier, producer_step, consumer_step, run_producer_loop, run_consumer_loop};

use std::thread;
use libc::{mlockall, MCL_CURRENT, MCL_FUTURE};

fn main() {
    // 1. Lock memory to prevent page faults in the hot path
    unsafe {
        if mlockall(MCL_CURRENT | MCL_FUTURE) != 0 {
            panic!("Failed to lock memory - check privileges.");
        }
    }

    // 2. Initialize Shared Data structures
    // Static allocation ensures cache alignment and predictable addresses
    static mut PRODUCER: ProducerCtrl = unsafe { std::mem::zeroed() };
    static mut CONSUMER: ConsumerCtrl = unsafe { std::mem::zeroed() };
    static mut RING_BUFFER: [u32; 65536] = [0; 65536];
    let MASK: u32 = 65535;

    // Thread 1: Producer on Core 0 (Handling interrupts/OS tasks)
    let producer_handle = thread::spawn(move || {
        set_affinity(0); // Pin to Core 0
        unsafe { run_producer_loop(&mut PRODUCER, &mut RING_BUFFER, MASK); }
    });

    // Thread 2: Consumer on Core 1 (Isolated, deterministic path)
    let consumer_handle = thread::spawn(move || {
        set_affinity(1); // Pin to Core 1
        unsafe { run_consumer_loop(&mut CONSUMER, &RING_BUFFER, MASK); }
    });

        // Benchmark loop
    let iterations = 1_000_000;

        // 1. Warm-up (No recording)
        for _ in 0..iterations {
            unsafe {
                producer_step(&mut PRODUCER, &mut RING_BUFFER, 0, MASK);
                consumer_step(&mut CONSUMER, &mut RING_BUFFER, MASK);
            }
        }

        // 2. Hot-path (Recording)
        for _ in 0..iterations {
            let (start, _) = read_tsc();
            unsafe {
                producer_step(&mut PRODUCER, &mut RING_BUFFER, 0, MASK);
                consumer_step(&mut CONSUMER, &mut RING_BUFFER, MASK);
            }
            let (end, _) = read_tsc();
            
            let diff = (end - start) as usize;
            let bucket = (diff / 10).min(999);
            unsafe { crate::lib::LATENCY_HISTOGRAM[bucket] += 1; }
        }

        // 3. Print report and EXIT naturally
        print_latency_report(iterations);
        

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();
}

// OS-specific pinning (Linux)
fn set_affinity(core_id: usize) {
    let mut cpuset = unsafe { std::mem::zeroed::<libc::cpu_set_t>() };
    unsafe {
        libc::CPU_ZERO(&mut cpuset);
        libc::CPU_SET(core_id, &mut cpuset);
        let pid = 0; // Current thread
        libc::sched_setaffinity(pid, std::mem::size_of::<libc::cpu_set_t>(), &cpuset);
    }
}