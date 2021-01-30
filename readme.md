
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
TSP_INITIAL_COORDS='6.5,8.5 7.5,8.5 8.5,8.5 7.5,8.51' cargo run --release -- spray 4 0.01
```

# Issues

For some reason local optimizations hurt the final solution. Because of this a technique must be
found which is able to avoid picking bad points, or a good definition for the order in which points
should be inserted.

Example failure: `envelope_triangle.tsp`.

A naive implementation will begin with the largest triangle and pick by shortest delta insertion weight.

Observe the failure (`-|.` represent path, `o` represents an unselected point, `X` is a point on path):

```
o                                o
                                  
                 o                
                                  
             o      o             
                                  
o                                o
```

Triangle:
```
o                                X
                            /    |
                 o     /         |
                 /               |
         /   o      o            |
   /                             |
X--------------------------------X
```

```
X--------------------------------X
|                                |
|                o               |
|                                |
|            o      o            |
|                                |
X--------------------------------X
```

```
X\                              /X
|         \              /       |
|                X               |
|                                |
|            o      o            |
|                                |
X--------------------------------X
```

Now we have a problem: moving to the ideal solution will be impossible to do in a single step
```
X\                              /X
|         \              /       |
|                X               |
|                                |
|            X      o            |
|      /          \              |
X/                              \X
```

```
X\                              /X
|         \              /       |
|                X               |
|                                |
|            X------X            |
|      /                  \      |
X/                              \X
```

The correct solution for this city is actually:

```
X--------------------------------X
|                                |
|               /X\              |
|             /    \             |
|           /X      X\           |
|     /                   \      |
X/                              \X
```

A major improvement designed using this test case was to track the center of the path, and insert the point furthest away.
This makes this test case work, but it does not scale to every TSP problem.


# Ideal solutions...

...for some problems and an outdated jalgo reference distance

Source: https://wwwproxy.iwr.uni-heidelberg.de/groups/comopt/software/TSPLIB95/STSP.html

Format is `city: ideal_sol jalgo_sol jalgo_ms`

All tests were run using the release binary at `./target/release/tsp-sol` using a 2014 macbook (`i5-4278U CPU @ 2.60GHz`, single-threaded, 8gb ram installed)

```
city:     ideal_sol      jalgo_sol jalgo_ms
berlin52:     7,542      8,521.919      4ms
rat783  :     8,806     11,156.645     12ms
pcb1173 :    56,892     73,337.836     21ms
rl5915  :   565,530    733,125.060    676ms
rl11849 :   923,288  1,198,670.090  5,485ms
st70    :       675        763.354     21ms

```

Quadratic regression for those numbers (x=size of city, y=time in ms) gives the function: `y = 128.1553 - 0.2493989*x + 0.00005912996*x^2`

Quadratic regression gives: `y = 3.81755 + 0.003054128*x + 0.000008696739*x^2 + 8.445805e-10*x^3 + 1.430081e-13*x^4`

Given the miniature size of the `x^3` and `x^4` coefficients I can feel confident my `O(N^2)` complexity is real.

# Misc


Good one-liner (s):

```
rm -rf views/* ; cargo run --release delta && ( ls -l1 views | wc -l )

rm -rf views/* ; TSP_INITIAL_COORDS='2.5,8.5 7.5,8.5 12.5,8.5 7.4,9.0' cargo run --release -- spray 4 0.1

```


# License

The code in this repository is under the GPLv2 license, see `LICENSE.txt` for details.
The auto-upgrade clause has been removed because your legal rights shouldn't have that sort of volatility.

