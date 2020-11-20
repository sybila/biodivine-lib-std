use std::thread;
use std::ops::Shl;
use std::time::SystemTime;

fn main() {
    let workers: usize = 4;
    let mut handles = Vec::new();
    for _ in 0..workers {
        handles.push(thread::spawn(|| {
            let randomizer: u64 = 31;
            let gb_count: usize = 1;
            let elements: usize = (1usize).shl(27usize).shl(gb_count - 1);
            let mut v: Vec<u64> = vec![0; elements]; // Reserve a 1GB buffer
            // Warm up caches!
            let mut number = 0;
            for i in 0..elements {
                v[i] = number;
                number = (number + 1) ^ randomizer;
            }
            let start = SystemTime::now();
            for it in 0..10 {
                println!("Iter {}", it);
                let mut number = 0;
                for i in 0..elements {
                    v[i] = number;
                    number = (number + 1) ^ randomizer;
                }
            }
            let elapsed = start.elapsed().unwrap();
            println!("Write 1GB of data into RAM in: {}ms", elapsed.as_millis() / ((gb_count * 10) as u128));
        }));
    }

    for _ in 0..workers {
        handles.pop().unwrap().join().unwrap();
    }
}