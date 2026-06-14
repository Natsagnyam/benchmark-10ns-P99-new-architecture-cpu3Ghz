use std::sync::atomic::{AtomicU32, Ordering};
use std::cell::UnsafeCell;

// --- Core Primitives: No Inline, No Macros ---

use std::mem::ManuallyDrop; // Add this for the union safety


#[repr(C)]
pub union Index {
    pub plain: u32,
    pub atomic: ManuallyDrop<UnsafeCell<AtomicU32>>,
}
/*
ManuallyDrop: This is a transparent wrapper. It generates zero machine code instructions. 
It purely serves to inform the Rust borrow checker that you are taking responsibility for the memory. 
It has no impact on your MESI protocol suppression or cache alignment.
*/

#[repr(C, align(64))]
pub struct ProducerCtrl {
    pub pad1: [u8; 124],
    pub tail: Index,
    pub pad2: [u8; 124],
}

#[repr(C, align(64))]
pub struct ConsumerCtrl {
    pub pad1: [u8; 124],
    pub head: Index,
    pub pad2: [u8; 124],
}

// --- Direct Assembly Primitives ---
pub fn read_tsc() -> (u64, u32) { // Return both TSC and Core ID
    let mut low: u32;
    let mut high: u32;
    let mut aux: u32;
    unsafe {
        core::arch::asm!(
            "rdtscp",
            out("eax") low,
            out("edx") high,
            out("ecx") aux,
            options(nostack, nomem, preserves_flags)
        );
    }
    (((high as u64) << 32) | (low as u64), aux)
}

pub fn publish_barrier() {
    unsafe { core::arch::asm!("sfence", options(nostack, nomem, preserves_flags)); }
}

pub fn acquire_barrier() {
    unsafe { core::arch::asm!("lfence", options(nostack, nomem, preserves_flags)); }
}

// --- Logic Implementation ---

pub fn producer_step<T>(ctrl: &mut ProducerCtrl, buffer: &mut [T], item: T, mask: u32) {
    let idx = (unsafe { ctrl.tail.plain } & mask) as usize;
    buffer[idx] = item;
    
    publish_barrier();
    
    unsafe { ctrl.tail.plain = ctrl.tail.plain.wrapping_add(1) };
}


/* Lifetime Annotations ('a): These are compile-time markers only. 
They exist solely to verify that you don't use the returned reference after the buffer has been destroyed. 
They do not appear in your final assembly output.*/
pub fn consumer_step<'a, T>(
    _ctrl: &mut ConsumerCtrl, 
    buffer: &'a [T], 
    mask: u32
) -> Option<&'a T> {
    // ... your logic ...
    None
}


pub fn run_producer_loop(ctrl: &mut ProducerCtrl, buffer: &mut [u32], mask: u32) {
    let iterations = 1_000_000; // Define a finite test run
    for _ in 0..iterations {        
        let (start, _aux) = read_tsc(); // Destructure to get the TSC (start) and ignore the core ID (_aux)

        producer_step(ctrl, buffer, 42, mask);

        let (end, _aux) = read_tsc(); // Destructure again for the end timestamp
        record_latency(start, end);   // Now both arguments are u64 as expected
    }
    // Only print once the hot path is complete
    print_histogram();
}


pub fn run_consumer_loop<const N: usize, T>(
    ctrl: &mut ConsumerCtrl, 
    buffer: &[T; N], 
    mask: u32
) {
    let mut local_head: u32 = 0;

    loop {
        // Record start of the polling cycle
        let (start, _aux) = read_tsc();

        // 1. Acquire Visibility
        let shared_head = unsafe { 
            (*ctrl.head.atomic.get()).load(Ordering::Acquire) 
        };

        if local_head == shared_head {
            // Optional: record the wait-time/poll-time here
            continue; 
        }

        // 2. Memory Barrier
        acquire_barrier();

        // 3. Process Data
        let idx = (local_head & mask) as usize;
        // ... process buffer[idx] ...

        // 4. Update head
        local_head = local_head.wrapping_add(1);
        unsafe { ctrl.head.plain = local_head };

        // Record end of the processing cycle
        let (end, _aux) = read_tsc();
        
        // record_latency(_start, _end);
    }
}

// Pre-allocate a large buffer for the latency histogram
// 1000 buckets, each representing 10 cycles (approx 2-3ns on 3.5GHz CPU)
const HISTOGRAM_SIZE: usize = 1000;
static mut LATENCY_HISTOGRAM: [u64; HISTOGRAM_SIZE] = [0; HISTOGRAM_SIZE];

pub fn record_latency(start: u64, end: u64) {
    let delta = end.wrapping_sub(start);
    // Bucket size: 10 cycles
    let bucket = (delta / 10) as usize;
    if bucket < HISTOGRAM_SIZE {
        unsafe { LATENCY_HISTOGRAM[bucket] += 1; }
    }
}

// Call this AFTER the loop finishes
pub fn print_histogram() {
    println!("--- Latency Histogram (10-cycle buckets) ---");
    unsafe {
        for (i, count) in LATENCY_HISTOGRAM.iter().enumerate() {
            if *count > 0 {
                println!("{} cycles: {} hits", i * 10, count);
            }
        }
    }
}