
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

# Triangle+1 or +2 multi-pattern-scan research
TSP_INITIAL_COORDS='4.0,9.0 10.0,9.0 7.0,4.0 ' cargo run --release -- pattern-scan 3 0.03

# Take 10 scans between the two cities (bottom 2 points stay, top point moves from left -> right)
TSP_INITIAL_COORDS='4.0,11.0 11.0,11.0 4.0,4.0 ' TSP_ENDING_COORDS='4.0,11.0 11.0,11.0 11.0,4.0 ' cargo run --release -- multi-pattern-scan 3 0.03 10 && mpv --loop-file=inf views/multi-pattern-scan.gif


# More detailed puzzle, scaled using magnify_coords.py

TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 6.0,8.50 9.0,8.50 12.0,3.0 12.0,12.0 7.5,6.5' cargo run --release -- pattern-scan 7 0.2

# Move point 6 from x=3.0 to x=12.0
TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 6.0,8.50 9.0,8.50 12.0,3.0 12.0,12.0 3.0,6.5' TSP_ENDING_COORDS='3.0,3.0 3.0,12.0 6.0,8.50 9.0,8.50 12.0,3.0 12.0,12.0 12.0,6.5' cargo run --release -- multi-pattern-scan 7 0.2 10 && mpv --loop-file=inf views/multi-pattern-scan.gif

# More detailed version of the same
TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 6.0,8.50 9.0,8.50 12.0,3.0 12.0,12.0 3.0,6.5' TSP_ENDING_COORDS='3.0,3.0 3.0,12.0 6.0,8.50 9.0,8.50 12.0,3.0 12.0,12.0 12.0,6.5' cargo run --release -- multi-pattern-scan 7 0.035 25 && mpv --loop-file=inf views/multi-pattern-scan.gif

# square with 5th point passing through center going left -> right
TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0 3.0,7.5' TSP_ENDING_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0 12.0,7.5' cargo run --release -- multi-pattern-scan 5 0.025 32

# 5-point envelope w/ moving biased 6th point left->right along the bottom of the graph
TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0 7.5,7.5 3.0,9' TSP_ENDING_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0 7.5,7.5 12.0,9' cargo run --release -- multi-pattern-scan 6 0.025 32


# Back to basics, 2 triangles w/ 3rd point moving left-right and up-down (between other 2 points)
TSP_INITIAL_COORDS='3.0,12.0 12.0,12.0 3.0,3.0 ' TSP_ENDING_COORDS='3.0,12.0 12.0,12.0 12.0,3.0 ' cargo run --release -- multi-pattern-scan 3 0.025 32 && mpv --loop-file=inf views/multi-pattern-scan.gif

TSP_INITIAL_COORDS='3.0,7.5 12.0,7.5 7.5,3.0 ' TSP_ENDING_COORDS='3.0,7.5 12.0,7.5 7.5,12.0 ' cargo run --release -- multi-pattern-scan 3 0.025 32 && mpv --loop-file=inf views/multi-pattern-scan.gif

USE_BRUTE_CACHE=f cargo run --release -- spray-pattern-search 3 0.025 500

USE_BRUTE_CACHE=f cargo run --release -- spray-pattern-search 4 0.05 100


# Dumb tiny triangle; do we see 3 divergent patterns at the peaks, or do they only appear w/
# a significant edge weight ratio differential?
# NB: 7.5 is x center of graph
USE_BRUTE_CACHE=f TSP_INITIAL_COORDS='6.5,7.0 7.5,7.0 7.0,6.5 ' cargo run --release -- pattern-scan 3 0.025

USE_BRUTE_CACHE=f TSP_INITIAL_COORDS='6.5,7.0 7.5,7.0 7.0,6.65 ' cargo run --release -- pattern-scan 3 0.025

# Square w/ biased left-right point moving
USE_BRUTE_CACHE=f TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0 3.0,9' TSP_ENDING_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0 12.0,9' cargo run --release -- multi-pattern-scan 5 0.025 64

USE_BRUTE_CACHE=f TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0' TSP_ENDING_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 3.0,12.0' cargo run --release -- multi-pattern-scan 4 0.025 64


# Current research following a very simple set of decision areas
# that are still complex enough to observe a backtracking-like behavior.
INCREMENT_NONCE_ON_ROW=f \
  USE_BRUTE_CACHE=f \
  TSP_INITIAL_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 12.0,3.0' \
  TSP_ENDING_COORDS='3.0,3.0 3.0,12.0 12.0,12.0 3.0,12.0' \
  cargo run --release -- multi-pattern-scan 4 0.049 32


# 2023-10-14 research, following/modeling 3d conics to
# match 2d parabolic shapes/area regions
TSP_INITIAL_COORDS='3.0,11.0 12.0,11.0 3.0,9.0 ' TSP_ENDING_COORDS='3.0,11.0 12.0,11.0 12.0,9.0 ' cargo run --release -- multi-pattern-scan 3 0.027 16

# Support fns
python -c "print(', '.join([ f'{x*0.5}' for x in range(-20, 20) ]))"
feh -d $( find views -iname '*-parabola.png' )



# 2023-10-19 research, exploring the 6 conic coefficients & attempting to fit the equation to perimiters of our solution region spaces
INITIAL_FORMULA="(-2279.5137 * x**2) + (2301.4355 * xy) + (-519.5718 * y**2) + (1271.6567 * x) + (-1471.1874 * y) + 2721.0815 = 0" COEFICIENT_MIN=-4000.0 COEFICIENT_MAX=4000.0 python scripts/conic_playground.py ./views/multi-pattern-scan-002-parabola.txt-6073b6-8c7a38-edge-points.json COEFICIENT_MIN=-4000.0 COEFICIENT_MAX=4000.0 graph_edge_points_file=./views/multi-pattern-scan-002-parabola.txt-6073b6-8c7a38-edge-points.json


# 2024-02-03 stuff
TSP_INITIAL_COORDS='3.0,11.0 12.0,11.0 5.0,9.0 ' TSP_ENDING_COORDS='3.0,11.0 12.0,11.0 7.0,9.0 ' cargo run --release -- multi-pattern-scan 3 0.027 4

# GPU env var values
PREF_GPU=print cargo run --release -- # prints all detected GPUs
PREF_GPU=none  cargo run --release -- # forces CPU even if a GPU is attached
PREF_GPU=intel cargo run --release -- # selects first GPU with "intel" in the name




```

# Research on city size & total optimal solutions observed

This was collected by running `spray-pattern-search` of size `city size` and scanning results for
the largest number of distinct tours with:

```bash
find views -mindepth 1 -maxdepth 1 -type d -exec sh -c "ls {} | wc -l " \;

# Print folder names for detailed inspections
find views -mindepth 1 -maxdepth 1 -type d -print -exec sh -c "ls {} | wc -l " \;

# Print max(numbers) for the lazy researcher
find views -mindepth 1 -maxdepth 1 -type d -exec sh -c "ls {} | wc -l " \; | sort -n | tail -n 1

```

```
city size, maximum optimal solutions seen (of >=100 random cities searched to detail 0.06)
3          3
4          6
5          9
6          12
7          15
8          20
9          ??
10         ??
11         ??

```

Fitting a 3rd-degree polynominal to this gives us roughly `0.2x^2 + 0.9x - 1.4`, not sure what is means but the `x^2` term
tells me I'm probably looking at a worst-case scenario of `n^2` possible optimal solutions to search going from `N` to `N+` points in a graph.




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
 - https://www.andrews.edu/~rwright/Precalculus-RLW/Text/07-05.html
 - https://math.sci.ccny.cuny.edu/document/Rotation+of+Axes


# Publishing/data sharing one-liners

```bash
rsync -v -r --size-only --links --delete /j/proj/tsp-sol/views/ /mnt/machome/miscellaneous/jeff-tsp-views

```


# License

The code in this repository is under the GPLv2 license, see `LICENSE.txt` for details.
The auto-upgrade clause has been removed because your legal rights shouldn't have that sort of volatility.

