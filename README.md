# Amdahl's Lie - Speed up parallel code beyond Amdahl's Law

This repository provides a proof-of-concept that shows how to break Amdahl's Law using caches. This tries to show that `n`-threaded applications can be more than `n` times as fast. This works for applications that can partition their work into memory slices.

This is by no means supposed to be a criticism of Amdahl's Law which is extremely useful in parallel programming.

## License

Licensed under an MIT license.