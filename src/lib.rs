#[derive(Clone, Copy)]
pub struct Config {
    pub num_bytes_per_section: usize,
    pub num_sections: usize,
    pub num_iterations: usize,
}

pub struct Request {
    pub start: usize,
    pub section: usize,
}

impl Config {
    pub fn total_bytes(self) -> usize {
        self.num_bytes_per_section * self.num_sections
    }
}

#[inline(never)]
fn handle_request(set: &[u8], start: usize, mask: usize, num_iterations: usize) -> u8 {
    let mut offset = start;
    let mut sum = 0u8;

    const LOAD_SPACING: usize = 0x200;

    for _ in 0..num_iterations {
        // Two Notes:
        // 1. We load here multiple times to increase the memory burden per loop. Especially with
        //    the multiplication. This is important.
        // 2. We space out the loads a bit further so that one request will probably not
        //    immediately cache the others (L1 cache-line sizes are usually not that big).
        let v1 = set[(offset + 0 * LOAD_SPACING) & mask];
        let v2 = set[(offset + 1 * LOAD_SPACING) & mask];
        let v3 = set[(offset + 2 * LOAD_SPACING) & mask];
        let v4 = set[(offset + 3 * LOAD_SPACING) & mask];

        sum = sum.wrapping_add(v1);
        sum = sum.wrapping_add(v2);
        sum = sum.wrapping_add(v3);
        sum = sum.wrapping_add(v4);

        // Multiplication, as opposed to addition, to make the memory accesses less predictable.
        //
        // For predictable memory accesses, the core detects the pattern and starts prefetching
        // memory.
        offset *= 791;
        offset &= mask;
    }

    sum
}

/// Gives a bitmask that will restrict a number to at most `n`.
///
/// The bitmask might not produce the range `0` to `n` but will produce a range `0` to `m` with `m
/// <= n`
fn num_to_bitmask(n: usize) -> usize {
    if n.is_power_of_two() {
        n - 1
    } else {
        (n.next_power_of_two() >> 1) - 1
    }
}

/// Handle all requests on a single thread in order of the requests
pub fn singlethreaded(set: &'static [u8], requests: &[Request], cfg: Config) -> Vec<u8> {
    let mut sums = Vec::with_capacity(requests.len());

    let mask = num_to_bitmask(cfg.num_bytes_per_section);

    for rq in requests {
        let start = rq.start;
        let section_start = rq.section * cfg.num_bytes_per_section;
        let sum = handle_request(&set[section_start..], start, mask, cfg.num_iterations);
        sums.push(sum);
    }

    sums
}

/// Handle all requests on a single thread batched by the request section
pub fn singlethreaded_batched(set: &'static [u8], requests: &[Request], cfg: Config) -> Vec<u8> {
    let mask = num_to_bitmask(cfg.num_bytes_per_section);

    let mut vec_of_sums: Vec<Vec<u8>> = vec![Vec::with_capacity(requests.len()); cfg.num_sections];

    for thread in 0..cfg.num_sections {
        for rq in requests {
            if rq.section != thread {
                continue;
            }

            let start = rq.start;
            let section_start = rq.section * cfg.num_bytes_per_section;
            let sum = handle_request(&set[section_start..], start, mask, cfg.num_iterations);
            vec_of_sums[thread].push(sum);
        }
    }

    let mut idxs = vec![0usize; cfg.num_sections];
    let mut sums = Vec::with_capacity(requests.len());

    for rq in requests {
        let idx = idxs[rq.section];
        sums.push(vec_of_sums[rq.section][idx]);
        idxs[rq.section] += 1;
    }

    sums
}

pub fn multithreaded(set: &'static [u8], requests: &[Request], cfg: Config) -> Vec<u8> {
    let mut queues: Vec<Vec<usize>> = vec![Vec::default(); cfg.num_sections];

    for rq in requests {
        queues[rq.section].push(rq.start);
    }

    let mut handles = Vec::with_capacity(cfg.num_sections);

    let mask = num_to_bitmask(cfg.num_bytes_per_section);

    for (i, queue) in queues.into_iter().enumerate() {
        handles.push(std::thread::spawn(move || {
            let start = i * cfg.num_bytes_per_section;
            let end = start + cfg.num_bytes_per_section;

            let pool_data_set = &set[start..end];

            let mut sums: Vec<u8> = Vec::with_capacity(queue.len());

            for prime in queue.into_iter() {
                let sum = handle_request(pool_data_set, prime, mask, cfg.num_iterations);
                sums.push(sum);
            }

            sums
        }));
    }

    let mut vec_of_sums = Vec::with_capacity(cfg.num_sections);

    for handle in handles.into_iter() {
        vec_of_sums.push(handle.join().unwrap());
    }

    let mut idxs = vec![0usize; cfg.num_sections];
    let mut sums = Vec::with_capacity(requests.len());

    for order in requests {
        let idx = idxs[order.section];
        sums.push(vec_of_sums[order.section][idx]);
        idxs[order.section] += 1;
    }

    sums
}

#[cfg(test)]
mod tests {
    use super::*;

    static PRIMES: [usize; 100] = [
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
        101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
        197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
        311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421,
        431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541,
    ];

    fn config() -> Config {
        Config {
            num_bytes_per_section: 256 * (1 << 10),
            num_sections: 4,
            num_iterations: 100_000,
        }
    }

    fn data_set() -> &'static [u8] {
        use rand::prelude::*;
        use rand_chacha::ChaCha8Rng;

        let cfg = config();

        let mut rng = ChaCha8Rng::seed_from_u64(0x1337);
        let mut data_set = Vec::with_capacity(cfg.num_bytes_per_section * cfg.num_sections);

        for _ in 0..cfg.num_bytes_per_section * cfg.num_sections {
            data_set.push(rng.gen());
        }

        data_set.leak()
    }

    fn orders() -> Vec<Request> {
        use rand::prelude::*;
        use rand_chacha::ChaCha8Rng;

        let cfg = config();

        const NUM_ORDERS: usize = 1_000;

        let mut rng = ChaCha8Rng::seed_from_u64(1337);
        let mut orders = Vec::with_capacity(NUM_ORDERS);

        for _ in 0..NUM_ORDERS {
            let start = PRIMES[rng.gen_range(0..100)] % cfg.num_bytes_per_section;
            let section = rng.gen_range(0..cfg.num_sections);

            orders.push(Request { start, section });
        }

        orders
    }

    #[test]
    fn single() {
        let start = std::time::SystemTime::now();

        let set = data_set();
        let orders = orders();

        singlethreaded(set, &orders, config());

        let duration = start.elapsed().unwrap();

        println!("Took: {}s", duration.as_secs_f32());
    }

    #[test]
    fn multi() {
        let start = std::time::SystemTime::now();

        let set = data_set();
        let orders = orders();

        multithreaded(set, &orders, config());

        let duration = start.elapsed().unwrap();

        println!("Took: {}s", duration.as_secs_f32());
    }

    #[test]
    fn equality() {
        let set = data_set();
        let orders = orders();

        let single = singlethreaded(set, &orders, config());
        let multi = multithreaded(set, &orders, config());

        assert_eq!(single, multi);
    }
}
