
# JAlgo

A novel Traveling Salesman algorithm in `O(N^2)` time, which aims to be a proper deterministic solution to all euclidian symmetric TS problems.

# Theory

This approach decomposes finding an ideal hamiltonian cycle into:

 - Begin with a path through any 3 points.
   - Observe this itself is a solution to TSP - for all points in path, is smallest hamiltonian
   - Also note any vity of size 3 or fewer is a no-op to solve TSP - the only path is the fastest
 - Pick a point from the city that is not in the path.
   - Insert this point into the path such that if (Path) is fastest hamiltonian, (Path+Point) will also be fastest hamiltonian.
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

# Issues

For some reason local optimizations hurt the final solution. Because of this a technique must be
found which is able to avoid picking bad points, or a good defenition for the order in which points
should be inserted.

Example failure: `envelope_triangle.tsp`.

A naieve implementation will begin with the largest triangle and pick by shortest delta insertion weight.

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

Format is `city: ideal_sol jalgo_sol jalso_ms`

All tests were run using the release binary at `./target/release/tsp-sol` using a 2014 macbook (`i5-4278U CPU @ 2.60GHz`, single-threaded, 8gb ram installed)

```
berlin52:   7,542      8,521.919      4ms
rat783  :   8,806     11,156.645     12ms
pcb1173 :  56,892     73,337.836     21ms
rl5915  : 565,530    733,125.060    676ms
rl11849 : 923,288  1,198,670.090  5,485ms


```



