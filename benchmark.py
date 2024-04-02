#!/usr/bin/env python

import subprocess
import sys

from enum import Enum
from random import randrange

class Variant(Enum):
    SINGLE = 0
    MULTI = 1
    BATCH = 2

    def to_string(self) -> str:
        match self:
            case Variant.SINGLE: return "single"
            case Variant.MULTI: return "multi"
            case Variant.BATCH: return "batch"

def run_benchmark(
    variant: Variant,
    num_bytes_per_section: int,
    num_sections: int,
    num_requests: int,
    seed: int,
) -> float:
    argv = [
        './target/release/amdahls-lie',
        variant.to_string(),
        str(num_bytes_per_section),
        str(num_sections),
        str(num_requests),
        str(seed),
    ]

    output = subprocess.run(argv, capture_output = True)
    output.check_returncode()

    output = output.stdout.decode()
    output = float(output.strip())

    return output

WARM_UPS = 0
REPEATS  = 2

def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <output file>")
        exit(2)

    output_path = sys.argv[1]

    f = open(output_path, "w")

    f.write(f"variant,num_bytes_per_section,num_sections,num_requests,seed,time\n")

    num_requests_cfgs = [7]
    num_sections_cfgs = [8]

    done = 0
    NUM_CONFIGURATIONS = len(num_requests_cfgs) * len(num_sections_cfgs)

    for num_requests_pow in num_requests_cfgs:
        num_requests = pow(10, num_requests_pow)

        for num_bytes_per_section in [250000]:
            for num_sections in num_sections_cfgs:
                print(f'Finished {done}/{NUM_CONFIGURATIONS}...\r', end = '')

                for i in range(WARM_UPS + REPEATS):
                    seed = randrange(1 << 32)

                    a = []

                    for variant in [Variant.SINGLE, Variant.MULTI, Variant.BATCH]:
                        time = run_benchmark(variant, num_bytes_per_section, num_sections, num_requests, seed)

                        a.append(time)

                        if i >= WARM_UPS:
                            f.write(f"{variant.to_string()},{num_bytes_per_section},{num_sections},{num_requests},{seed},{time}\n")
                            f.flush()

                done += 1

    print('')
    f.close()

    print(f'Speed-Up: {a[0] / a[1]}x')

if __name__ == "__main__":
    main()