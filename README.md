# Amdahl's Lie - Speeding up code beyond the theoretical limit

This repository provides a proof-of-concept that shows how to break Amdahl's Law using caches. This tries to show that `n`-threaded applications can be more than `n` times as fast. This works for applications that can partition their work into memory slices.

This is by no means supposed to be a criticism of Amdahl's Law which is extremely useful in parallel programming.

## Explanation

This program is just a proof-of-concept to idea behind breaking Amdahl's Law.

The program is divided into a `lib.rs` and `main.rs` as to simplify benchmarking. The `lib.rs` contains all the interesting logic. There are three implementations: `singlethreaded`, `singlethreaded_batched` and `multithreaded`. To each of these implementations, you pass (i) a `set` of data, (ii) a list of `requests`, and (iii) a configuration (`cfg`). Each implementation will then call `handle_request` for each request and return a vector with all the results.

The `handle_request` function aims to do a large calculation over data that is present in the `set`. This is not that weird of a situation as the same might happen in for example a database. In this`handle_request` implementation, we unpredictably load a bunch of bytes and create a sum over them. This range of bytes that we work on is dependent on the `request.section`.

## License

Licensed under an MIT license.