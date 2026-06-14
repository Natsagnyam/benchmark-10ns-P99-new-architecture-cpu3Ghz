use std::sync::atomic::{AtomicU32, Ordering};
use std::cell::UnsafeCell;

// Rule: Union-based tail/head for zero MESI broadcast on fast path
#[repr(C)]
pub union Index {
    pub plain: u32,
    pub atomic: UnsafeCell<AtomicU32>,
}

// Rule: 128-byte alignment (spans 2 cache lines) for isolation
#[repr(C, align(64))]
pub struct ProducerCtrl {
    pub pad1: [u8; 124], // Spans 2 full cache lines
    pub tail: Index,     // Union tail
    pub pad2: [u8; 124], // Isolation
}

#[repr(C, align(64))]
pub struct ConsumerCtrl {
    pub pad1: [u8; 124], // Spans 2 full cache lines
    pub head: Index,     // Union head
    pub pad2: [u8; 124], // Isolation
}

// Decoupled Ring Buffer storage for zero-copy data locality
#[repr(C)]
pub struct NatskaEngine_ringBuffer<const N: usize, T> {
    pub buffer: [T; N], 
    pub size_mask: u32,
}

// Decoupled Controller structs to prevent compiler-induced object merging
#[repr(C)]
pub struct NatskaEngine_thread1 {
    pub producer: ProducerCtrl,
}

#[repr(C)]
pub struct NatskaEngine_thread2 {
    pub consumer: ConsumerCtrl,
}

/*
use std::arch::x86_64::_rdtscp;

#[inline(always)]
pub fn read_tsc() -> u64 {
    let mut aux = 0;
    // Rdtscp is a serializing instruction on x86-64 
    // It captures the TSC and forces all previous ops to finish
    unsafe { _rdtscp(&mut aux) }
} */

pub fn read_tsc() -> u64 {
    let mut low: u32;
    let mut high: u32;
    let mut aux: u32;

    unsafe {
        // rdtscp instruction:
        // Reads TSC, stores in EDX:EAX, and stores TSC_AUX (core ID) in ECX.
        // It is a serializing instruction: no preceding instructions 
        // can be reordered after it, and no subsequent instructions 
        // can be reordered before it.
        core::arch::asm!(
            "rdtscp",
            out("eax") low,
            out("edx") high,
            out("ecx") aux,
            options(nostack, nomem, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64)
}


pub fn run_producer_loop(producer_ctrl: &mut ProducerCtrl, buffer: &mut [Packet]) {
    loop { // Rule: Hard-pinned while(1) loop
        let start = read_tsc();
        
        // 1. Logic processing (no syscalls)
        // 2. Write to buffer[tail & mask]
        // 3. Update tail.plain
        unsafe {
            producer_ctrl.tail.plain += 1;
        }
        
        let end = read_tsc();
        // Record latency histogram using linear 10ns buckets
    }
}


use std::arch::x86_64::{_mm_sfence, _mm_lfence};



// Rule: No inline, no macros, no preprocessor.
// Using core::arch::asm for direct, predictable instruction emission.

pub fn publish_barrier() {
    unsafe {
        // sfence: Explicitly emits the store fence instruction 
        // without compiler-added inlining logic.
        core::arch::asm!("sfence", options(nostack, nomem, preserves_flags));
    }
}

pub fn acquire_barrier() {
    unsafe {
        // lfence: Explicitly emits the load fence instruction.
        core::arch::asm!("lfence", options(nostack, nomem, preserves_flags));
    }
}

// Producer: The Zero-MESI Hot Path
pub fn producer_step<T>(ctrl: &mut ProducerCtrl, buffer: &mut [T], item: T, mask: u32) {
    let idx = (unsafe { ctrl.tail.plain } & mask) as usize;
    buffer[idx] = item;
    
    publish_barrier(); // Ensure data is visible before tail update
    
    unsafe { ctrl.tail.plain = ctrl.tail.plain.wrapping_add(1) };
}

// Consumer: The Safe Visibility Path
pub fn consumer_step<T>(ctrl: &mut ConsumerCtrl, buffer: &[T], mask: u32) -> Option<&T> {
    let head = unsafe { (*ctrl.head.atomic.get()).load(Ordering::Relaxed) };
    
    acquire_barrier(); // Ensure we see the data AFTER reading the head
    
    // ... logic to read buffer[head & mask]
    None 
}

