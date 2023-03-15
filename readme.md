
# JAlgo

A novel Traveling Salesman algorithm in `O(N^2)` time, which aims to be a proper deterministic solution to all euclidean symmetric TS problems.

**Edit** This algorithm it not so novel as the author originally thought, and falls into a class of TSP heuristic algorithms known as "Insertion Methods". The author maintains the mindset that this algorithm can go beyond a heuristic to provide a proper solution.

# Theory

This approach decomposes finding an ideal Hamiltonian cycle into:

 - Begin with a path through any 3 points.
   - Observe this itself is a solution to TSP - for all points in path, is smallest Hamiltonian
   - Also note any city of size 3 or fewer is a no-op to solve TSP - the only path is the fastest
 - Pick a point from the city that is not in the path.
   - Insert this point into the path such that if (Path) is fastest Hamiltonian, (Path+Point) will also be fastest Hamiltonian.
   - We can iterate all existing edges in path and insert at the smallest change (smallest of `-existing_weight+new_left_edge+new_right_edge`)
 - When all points are inserted in path, you have TSP solution for all given points

# Complexity

 - Iterating all nodes (from 3 -> N) = N
   - Iterate all existing edges (N-uninserted) = N

yields complexity of `O(N*N) = O(N^2)`

# Testing

You will need rust installed. Root access is _not_ required, you can install the toolchain under `~/.local/` by running

```bash
curl https://sh.rustup.rs -sSf | sh
# Remember to update your $PATH
```

Test the algorithm itself on a known tsp problem
```bash
cargo run --release tsp/berlin52.tsp
# or for performance tesing, do a `cargo build --release` and then
time ./target/release/tsp-sol tsp/berlin52.tsp
```

Test jalgo against the standard brute force approach for randomly generated cities:
```bash
cargo run --release delta
# ./views will be populated with images of steps whenever the two algorithm solutions differ
```

Setup a pre-defined city + spray it (add a point at each image pixel + print if insertion produced a Hamiltonian cycle)
```
RUSTFLAGS='-C target-cpu=native' TSP_INITIAL_COORDS='6.5,8.5 7.5,8.5 8.5,8.5 7.5,8.51' cargo run --release -- spray 4 0.01
```

# Ideal solutions...

...for some problems and an outdated jalgo reference distance

Source: https://wwwproxy.iwr.uni-heidelberg.de/groups/comopt/software/TSPLIB95/STSP.html

Source: http://elib.zib.de/pub/mp-testdata/tsp/tsplib/stsp-sol.html

Format is `city: ideal_sol jalgo_sol jalgo_ms`

All tests were run using the release binary at `./target/release/tsp-sol` using a Thinkpad t490 (`i7-8565U CPU @ 1.80GHz`, single-threaded, 16gb ram installed)

```
city:     ideal_sol      jalgo_sol   jalgo_ms
berma14 :    30.878         30.878        34ms
berlin52:     7,542      7,782.985        39ms
st70    :       675        687.916       119ms
rat99   :     1,211      1,250.809       484ms
rat783  :     8,806      9,324.545 1,938,490ms (32min 18s)
pcb1173 :    56,892     69,430.580         ?ms *untested with latest algo
rl5915  :   565,530    733,125.060         ?ms *untested with latest algo
rl11849 :   923,288  1,198,670.090         ?ms *untested with latest algo

```

TODO do quadratic regression on test plots as evidence of complexity factor.

```bash
# Nifty test one-liner
for t in berlin52 st70 rat783 pcb1173 ; do echo "Running $t" ; time ./target/release/tsp-sol tsp/$t.tsp ; done
```

# Misc


Good one-liner (s):

```bash
rm -rf views/* ; cargo run --release delta && ( ls -l1 views | wc -l )

rm -rf views/* ; TSP_INITIAL_COORDS='2.5,8.5 7.5,8.5 12.5,8.5 7.4,9.0' cargo run --release -- spray 4 0.1

rm -rf views/* ; TSP_INITIAL_COORDS='4.23,4.87 7.16,7.70 2.79,7.70 8.24,3.89 1.08,0.57 1.84,8.72 ' cargo run --release -- spray 6 0.9

# Run until we fail then selectively spray the first 7 cities
mkdir -p views/selective/ ; rm -rf views/selective/* ; cargo run --release selective && source views/selective/node-coordinates-env.txt && export TSP_INITIAL_COORDS=$TSP_INITIAL_COORDS && echo "TSP_INITIAL_COORDS=$TSP_INITIAL_COORDS" && cargo run --release -- spray 7 0.19


TSP_INITIAL_COORDS='5.79,5.22 7.01,9.88 6.61,9.35 9.53,8.49 7.48,8.06 6.44,8.85 5.43,8.73 5.27,9.86' cargo run --release -- spray 8 0.19

TSP_INITIAL_COORDS='8.20,7.28 7.11,6.85 6.71,7.18 9.34,8.21 8.67,5.21 6.09,5.51 5.11,7.05' cargo run --release -- spray 7 0.02


TSP_INITIAL_COORDS='7.13,5.80 7.17,7.57 5.90,7.68 7.91,5.98 8.59,5.42 7.14,6.45 6.52,8.38 9.23,5.49 ' cargo run --release -- spray 8 0.5


TSP_INITIAL_COORDS='6.00,6.75 7.25,7.44 6.72,7.41 6.24,8.49 7.74,8.15 8.92,8.45 5.86,9.41 8.30,5.51 ' cargo run --release -- spray 8 0.5


TSP_INITIAL_COORDS='5.86,7.10 6.99,7.07 5.61,7.32 5.46,6.08 7.99,7.20 6.81,8.98 6.06,7.16 6.05,6.73 9.89,5.84 ' cargo run --release -- spray 9 0.5



TSP_INITIAL_COORDS='8.62,5.06 8.45,6.15 7.67,5.18 9.33,9.81 5.18,6.28 7.50,5.49 9.62,5.43 7.61,5.30 7.83,6.15 ' cargo run --release -- spray 9 0.1


TSP_INITIAL_COORDS='6.28,5.48 7.58,7.49 9.75,6.73 7.84,6.12 8.45,7.94 5.90,5.18 7.30,7.80 9.42,7.22 7.24,5.64 5.48,5.32 8.15,5.12 ' cargo run --release -- spray 11 0.1


TSP_INITIAL_COORDS='4.00,4.00 4.00,6.00 5.00,5.17 6.00,5.17 7.00,4.00 7.00,6.00 5.50,4.88 ' cargo run --release -- pattern-scan 7 0.02



```

# Performance profiling

 - Install `perf` (see your OS's package manager for details)
 - Use cargo to install `cargo-flamegraph`: `cargo install flamegraph`
 - Run a release binary w/ to generate a flamegraph:

```bash
# May not always be necessary, see https://lwn.net/Articles/696216/ for details
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid

RUSTFLAGS='-C target-cpu=native' TSP_INITIAL_COORDS='5.79,5.22 7.01,9.88 6.61,9.35 9.53,8.49 7.48,8.06 6.44,8.85 5.43,8.73 5.27,9.86' cargo flamegraph -o /tmp/graph.svg -- spray 8 0.09

firefox /tmp/graph.svg


```

# Further research ideas

 - https://docs.rs/packed_simd/latest/packed_simd/index.html
    - Possible low-level performance bump?



# License

The code in this repository is under the GPLv2 license, see `LICENSE.txt` for details.
The auto-upgrade clause has been removed because your legal rights shouldn't have that sort of volatility.

