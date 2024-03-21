use std::io::stdout;
use std::sync::mpsc::channel;

#[derive(Clone, Copy)]
pub struct Config {
    pub num_bytes_per_thread: usize,
    pub num_threads: usize,
    pub num_iterations: usize,
}

pub struct Order {
    pub prime: usize,
    pub section: usize,
}

impl Config {
    pub fn total_bytes(self) -> usize {
        self.num_bytes_per_thread * self.num_threads
    }
}

#[inline(never)]
fn order_loop(set: &[u8], prime: usize, mask: usize, num_iterations: usize) -> u8 {
    let mut offset = 0usize;
    let mut sum = 0u8;
    
    // let mut stdout = stdout().lock();

    for _ in 0..num_iterations {
        let value = set[offset & mask];

        // use std::io::Write;
        //
        // writeln!(stdout, "offset = {offset}").unwrap();
        // writeln!(stdout, "value = {value}").unwrap();
        //
        sum = sum.wrapping_add(value);

        // Prevent simple prefetching algorithms from detecting the access pattern
        offset = offset.wrapping_add(prime);
        offset = offset.rotate_left(3) ^ 0xF;
        offset = offset.wrapping_add(0x4242_1337);
    }

    sum
}

pub fn single_thread(set: &'static [u8], orders: &[Order], cfg: Config) -> Vec<u8> {
    let mut sums = Vec::with_capacity(orders.len());

    let mask = (cfg.num_bytes_per_thread.next_power_of_two() >> 1) - 1;

    for order in orders {
        let section_start = order.section * cfg.num_bytes_per_thread;
        let sum = order_loop(&set[section_start..], order.prime, mask, cfg.num_iterations);
        sums.push(sum);
    }

    sums
}

pub fn multi_thread(set: &'static [u8], orders: &[Order], cfg: Config) -> Vec<u8> {
    let mut queues: Vec<Vec<(usize, usize)>> = vec![Vec::default(); cfg.num_threads];

    for (order_idx, order) in orders.iter().enumerate().rev() {
        queues[order.section].push((order_idx, order.prime));
    }

    let mut handles = Vec::with_capacity(cfg.num_threads);

    let mask = (cfg.num_bytes_per_thread.next_power_of_two() >> 1) - 1;

    for (i, queue) in queues.into_iter().enumerate() {
        handles.push(std::thread::spawn(move || {
            let start = i * cfg.num_bytes_per_thread;
            let end = start + cfg.num_bytes_per_thread;

            let pool_data_set = &set[start..end];

            queue
                .into_iter()
                .map(|(order_idx, prime)| {
                    let sum = order_loop(pool_data_set, prime, mask, cfg.num_iterations);
                    (order_idx, sum)
                })
                .collect::<Vec<(usize, u8)>>()
        }));
    }

    let mut sums = Vec::with_capacity(orders.len());

    for handle in handles.into_iter() {
        sums.extend_from_slice(&handle.join().unwrap());
    }

    sums.sort_by_key(|(idx, _)| *idx);
    let sums = sums.into_iter().map(|(_, v)| v).collect();

    sums
}

static PRIMES: [usize; 100] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
    311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421,
    431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541,
];

fn config() -> Config {
    Config {
        num_bytes_per_thread: 256 * (1 << 10),
        num_threads: 4,
        num_iterations: 100_000,
    }
}

fn data_set() -> &'static [u8] {
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    let cfg = config();

    let mut rng = ChaCha8Rng::seed_from_u64(0x1337);
    let mut data_set = Vec::with_capacity(cfg.num_bytes_per_thread * cfg.num_threads);

    for _ in 0..cfg.num_bytes_per_thread * cfg.num_threads {
        data_set.push(rng.gen());
    }

    data_set.leak()
}

fn orders() -> Vec<Order> {
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    let cfg = config();

    const NUM_ORDERS: usize = 1_000;

    let mut rng = ChaCha8Rng::seed_from_u64(1337);
    let mut orders = Vec::with_capacity(NUM_ORDERS);

    for _ in 0..NUM_ORDERS {
        let prime = PRIMES[rng.gen_range(0..100)] % cfg.num_bytes_per_thread;
        let section = rng.gen_range(0..cfg.num_threads);

        orders.push(Order { prime, section });
    }

    orders
}

#[test]
fn single() {
    let start = std::time::SystemTime::now();

    let set = data_set();
    let orders = orders();

    single_thread(set, &orders, config());

    let duration = start.elapsed().unwrap();

    println!("Took: {}s", duration.as_secs_f32());
}

#[test]
fn multi() {
    let start = std::time::SystemTime::now();

    let set = data_set();
    let orders = orders();

    multi_thread(set, &orders, config());

    let duration = start.elapsed().unwrap();

    println!("Took: {}s", duration.as_secs_f32());
}

#[test]
fn equality() {
    let set = data_set();
    let orders = orders();

    let single = single_thread(set, &orders);
    let multi = multi_thread(set, &orders);

    assert_eq!(single, multi);
}
