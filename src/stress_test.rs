use dashmap::DashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const ITERATIONS: u32 = 1_000_000;

fn main() {
    let set = Arc::new(DashSet::new());
    let thread_set = set.clone();
    let handle = thread::spawn(move || {
        let start = Instant::now();
        let mut durations = Vec::new();
        for i in 0..ITERATIONS {
            let iteration_start = Instant::now();
            thread_set.insert(i);
            durations.push(iteration_start.elapsed());
            // Make sure the other thread has a chance to get in the way.
            if i % 1000 == 0 {
                thread::sleep(Duration::from_micros(100));
            }
        }
        (start.elapsed(), durations)
    });
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    thread::spawn(move || loop {
        for i in 0..ITERATIONS {
            set.insert(i + ITERATIONS);
        }
        if thread_stop.load(Ordering::Relaxed) {
            break;
        }
    });
    let (elapsed, mut durations) = handle.join().unwrap();
    stop.store(true, Ordering::Relaxed);

    durations.sort();
    let percentile = |p: f64| -> Duration { durations[(ITERATIONS as f64 * p) as usize] };
    println!("Took: {:?}. Min iteration: {:?}. 10% iteration: {:?}. 50% iteration: {:?}. 90% iteration: {:?}. 99% iteration: {:?}. Max iteration: {:?}.",
        elapsed, durations[0], percentile(0.1), percentile(0.5), percentile(0.9), percentile(0.99), durations[(ITERATIONS - 1) as usize]);
}
