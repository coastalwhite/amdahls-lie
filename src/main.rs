use amdahls_lie::{Config, Request};

struct Args {
    subtask: String,
    cfg: Config,
    num_requests: usize,
    seed: u64,
}

fn take_args() -> Result<Args, String> {
    let mut args = std::env::args();

    args.next().unwrap();

    let subtask = args.next().ok_or("No subtask given".to_string())?;
    let num_bytes_per_section = args
        .next()
        .ok_or("No num_bytes_per_thread given".to_string())?;
    let num_sections = args.next().ok_or("No num_threads given".to_string())?;
    let num_requests = args.next().ok_or("No num_orders given".to_string())?;
    let seed = args.next().ok_or("No seed given".to_string())?;

    let num_bytes_per_section: usize = num_bytes_per_section
        .parse()
        .map_err(|_| "Invalid num_bytes_per_thread")?;
    let num_sections: usize = num_sections.parse().map_err(|_| "Invalid num_threads")?;
    let num_requests: usize = num_requests.parse().map_err(|_| "Invalid num_orders")?;
    let seed: u64 = seed.parse().map_err(|_| "Invalid seed")?;

    let cfg = Config {
        num_bytes_per_section,
        num_sections,
    };

    Ok(Args {
        subtask,
        cfg,
        num_requests,
        seed,
    })
}

fn main() {
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    let args = take_args().unwrap_or_else(|err| {
        eprintln!("Usage: amdahls_lie <single/multi/batch> <num_bytes_per_section> <num_sections> <num_requests> <seed>");
        eprintln!("- `single`: Single-Threaded with requests handled in-order");
        eprintln!("- `multi`:  Multi-Threaded (one thread per section)");
        eprintln!("- `batch`:  Single-Threaded with requests batched by section");
        eprintln!("");
        eprintln!("{err}");
        std::process::exit(2);
    });

    let Args {
        subtask,
        cfg,
        num_requests,
        seed,
    } = args;

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut data_set = Vec::with_capacity(cfg.total_bytes());
    let mut requests = Vec::with_capacity(num_requests);

    for _ in 0..cfg.total_bytes() {
        data_set.push(rng.gen());
    }

    for _ in 0..num_requests {
        let start = rng.gen();
        let section = rng.gen_range(0..cfg.num_sections);

        requests.push(Request { start, section });
    }

    let set = data_set.leak();
    let requests = &requests;

    let start = std::time::SystemTime::now();

    match &subtask[..] {
        "single" => {
            amdahls_lie::singlethreaded(set, requests, cfg);
        }
        "batch" => {
            amdahls_lie::singlethreaded_batched(set, requests, cfg);
        }
        "multi" => {
            amdahls_lie::multithreaded(set, requests, cfg);
        }
        _ => {
            eprintln!("Invalid Task: '{subtask}'!");
            std::process::exit(2);
        }
    }

    let duration = start.elapsed().unwrap();

    println!("{}", duration.as_secs_f32());
}
