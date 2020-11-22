use std::thread;
use std::ops::Shl;
use std::time::SystemTime;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::sync::Arc;

// Used to shuffle values a bit so the processor does not know what's going on.
const RANDOMIZER: u64 = 97;
// Used to randomize array indices during memory access.
const LARGE_PRIME: usize = 1073676287;

/// Build a buffer of (relatively) pseudo-random initial values for each thread.
fn allocate_buffers(workers: usize, elements: usize) -> Vec<Vec<u64>> {
    let mut number = 0;
    return (0..workers).map(|_| {
        let mut buffer = vec![0u64; elements];
        for i in 0..buffer.len() {
            buffer[i] = number;
            number = (number + buffer[i]) ^ RANDOMIZER;
        }
        buffer
    }).collect();
}

/// Translates transferred bytes per time to bandwidth in GB/s.
fn transfer_time_to_bandwidth(transferred_bytes: u128, elapsed_ms: u128) -> f64 {
    return ((transferred_bytes / elapsed_ms) as f64) / 1_000_000.0;
}

fn sequential_read_benchmark(workers: usize) {
    // Reserve a buffer for each worker:
    let elements: usize = (1usize).shl(26usize);    // ~500MB of 8byte values
    let buffers = allocate_buffers(workers, elements);
    // Perform read measurements:
    let atomic_counter = Arc::new(AtomicUsize::new(0));
    let start = SystemTime::now();
    buffers.into_iter()
        .map(|mut buffer| {
            let my_counter = atomic_counter.clone();
            thread::spawn(move || {
                let mut number = 0;
                while my_counter.load(Ordering::SeqCst) < (1_000_000_000 * workers) {   // ~8GB/worker
                    for i in 0..buffer.len() {
                        number = (number + buffer[i]) ^ RANDOMIZER;
                    }
                    my_counter.fetch_add(buffer.len(), Ordering::SeqCst);
                }
                buffer[0] = number; // make sure the number is not optimized away...
                buffer  // return buffer for future use...
            })
        })
        .collect::<Vec<JoinHandle<Vec<u64>>>>()
        .into_iter()
        .for_each(|handle| { handle.join().unwrap(); });
    println!();
    let bandwidth = transfer_time_to_bandwidth(
        8 * (atomic_counter.load(Ordering::SeqCst) as u128),
        start.elapsed().unwrap().as_millis()
    );
    println!("[SEQUENTIAL READ] {:.2} GB/s using {} thread(s).", bandwidth, workers);
}

fn sequential_read_write_benchmark(workers: usize) {
    // Reserve a buffer for each worker:
    let elements: usize = (1usize).shl(26usize);    // ~500MB of 8byte values
    let buffers = allocate_buffers(workers, elements);
    // Perform read measurements:
    let atomic_counter = Arc::new(AtomicUsize::new(0));
    let start = SystemTime::now();
    buffers.into_iter()
        .map(|mut buffer| {
            let my_counter = atomic_counter.clone();
            thread::spawn(move || {
                let mut number = 0;
                while my_counter.load(Ordering::SeqCst) < (1_000_000_000 * workers) {   // ~8GB/worker
                    for i in 0..buffer.len() {
                        buffer[i] = number;
                        number = (number + buffer[i]) ^ RANDOMIZER;
                    }
                    my_counter.fetch_add(buffer.len(), Ordering::SeqCst);
                }
                buffer[0] = number; // make sure the number is not optimized away...
                buffer  // return buffer for future use...
            })
        })
        .collect::<Vec<JoinHandle<Vec<u64>>>>()
        .into_iter()
        .for_each(|handle| { handle.join().unwrap(); });
    let bandwidth = transfer_time_to_bandwidth(
        // 2 * since every iteration performs one read and one write
        2 * 8 * (atomic_counter.load(Ordering::SeqCst) as u128),
        start.elapsed().unwrap().as_millis()
    );
    println!("[SEQUENTIAL READ/WRITE] {:.2} GB/s using {} thread(s).", bandwidth, workers);
}

fn random_read_benchmark(workers: usize) {
    // Reserve a buffer for each worker:
    let elements: usize = (1usize).shl(26usize);    // ~500MB of 8byte values
    let buffers = allocate_buffers(workers, elements);
    // Perform read measurements:
    let atomic_counter = Arc::new(AtomicUsize::new(0));
    let start = SystemTime::now();
    buffers.into_iter()
        .map(|mut buffer| {
            let my_counter = atomic_counter.clone();
            thread::spawn(move || {
                let mut number = 0;
                let mut index = 0;
                let buffer_len = buffer.len();
                while my_counter.load(Ordering::SeqCst) < (100_000_000 * workers) {   // ~0.8GB/worker
                    for _ in 0..buffer.len() {
                        // this should try all indices, but in pseudo-random order
                        index = (index + LARGE_PRIME) % buffer_len;
                        number = (number + buffer[index]) ^ RANDOMIZER;
                    }
                    my_counter.fetch_add(buffer.len(), Ordering::SeqCst);
                }
                buffer[0] = number; // make sure the number is not optimized away...
                buffer  // return buffer for future use...
            })
        })
        .collect::<Vec<JoinHandle<Vec<u64>>>>()
        .into_iter()
        .for_each(|handle| { handle.join().unwrap(); });
    let bandwidth = transfer_time_to_bandwidth(
        8 * (atomic_counter.load(Ordering::SeqCst) as u128),
        start.elapsed().unwrap().as_millis()
    );
    println!("[RANDOM READ] {:.2} GB/s using {} thread(s).", bandwidth, workers);
}

fn random_read_write_benchmark(workers: usize) {
    // Reserve a buffer for each worker:
    let elements: usize = (1usize).shl(26usize);    // ~500MB of 8byte values
    let buffers = allocate_buffers(workers, elements);
    // Perform read measurements:
    let atomic_counter = Arc::new(AtomicUsize::new(0));
    let start = SystemTime::now();
    buffers.into_iter()
        .map(|mut buffer| {
            let my_counter = atomic_counter.clone();
            thread::spawn(move || {
                let mut number = 0;
                let mut index = 0;
                let buffer_len = buffer.len();
                while my_counter.load(Ordering::SeqCst) < (100_000_000 * workers) {   // ~0.8GB/worker
                    for _ in 0..buffer.len() {
                        // this should try all indices, but in pseudo-random order
                        buffer[index] = number;
                        index = (index + LARGE_PRIME) % buffer_len;
                        number = (number + buffer[index]) ^ RANDOMIZER;
                    }
                    my_counter.fetch_add(buffer.len(), Ordering::SeqCst);
                }
                buffer[0] = number; // make sure the number is not optimized away...
                buffer  // return buffer for future use...
            })
        })
        .collect::<Vec<JoinHandle<Vec<u64>>>>()
        .into_iter()
        .for_each(|handle| { handle.join().unwrap(); });
    let bandwidth = transfer_time_to_bandwidth(
        2 * 8 * (atomic_counter.load(Ordering::SeqCst) as u128),
        start.elapsed().unwrap().as_millis()
    );
    println!("[RANDOM READ/WRITE] {:.2} GB/s using {} thread(s).", bandwidth, workers);
}

fn main() {
    // Read number of parallel workers from first command line argument.
    let workers: usize = std::env::args().skip(1).next().unwrap_or("1".to_string()).parse().unwrap();
    sequential_read_benchmark(workers);
    sequential_read_write_benchmark(workers);
    random_read_benchmark(workers);
    random_read_write_benchmark(workers);
}