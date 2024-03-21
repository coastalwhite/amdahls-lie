use amdahls_lie::{Config, Order};

static PRIMES: [usize; 100] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
    311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421,
    431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541,
];

fn main() {
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    let mut args = std::env::args();

    args.next().unwrap();

    let subtask = args.next().unwrap();
    let num_bytes_per_thread: usize = args.next().unwrap().parse().unwrap();
    let num_threads: usize = args.next().unwrap().parse().unwrap();
    let num_iterations: usize = args.next().unwrap().parse().unwrap(); 
    let num_orders: usize = args.next().unwrap().parse().unwrap(); 

    let config = Config {
        num_bytes_per_thread,
        num_threads,
        num_iterations,
    };

    let mut rng = ChaCha8Rng::seed_from_u64(0x1337);
    let mut data_set = Vec::with_capacity(config.num_bytes_per_thread * config.num_threads);
    let mut orders = Vec::with_capacity(num_orders);

    for _ in 0..config.total_bytes() {
        data_set.push(rng.gen());
    }

    for _ in 0..num_orders {
        let prime = PRIMES[rng.gen_range(0..100)] % config.num_bytes_per_thread;
        let section = rng.gen_range(0..config.num_threads);

        orders.push(Order { prime, section });
    }

    let set = data_set.leak();
    let orders = &orders;

    let start = std::time::SystemTime::now();

    match &subtask[..] {
        "single" => {
            amdahls_lie::single_thread(set, orders, config);
        }
        "multi" => {
            amdahls_lie::multi_thread(set, orders, config);
        }
        _ => {
            eprintln!("Invalid Task: '{subtask}'!");
            std::process::exit(1);
        }
    }

    let duration = start.elapsed().unwrap();

    println!("Took: {}s", duration.as_secs_f32());
}
