// src/main.rs
mod lib; // This tells the compiler to include your structures/primitives

use crate::lib::{ProducerCtrl, ConsumerCtrl, Index, read_tsc, publish_barrier, acquire_barrier, producer_step, consumer_step, run_producer_loop, run_consumer_loop};

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
    let mask: u32 = 65535;

    // 3. Spawn and Pin Threads
    // Thread 1: Producer on Core 1
    let producer_handle = thread::spawn(move || {
        set_affinity(1);
        unsafe { run_producer_loop(&mut PRODUCER, &mut RING_BUFFER, mask); }
    });

    // Thread 2: Consumer on Core 2
    let consumer_handle = thread::spawn(move || {
        set_affinity(2);
        unsafe { run_consumer_loop(&mut CONSUMER, &RING_BUFFER, mask); }
    });

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