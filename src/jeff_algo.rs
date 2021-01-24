/**
 *  tsp-sol - an experimental environment for traveling salesman solution analysis
 *  Copyright (C) 2019  Jeffrey McAteer <jeffrey.p.mcateer@gmail.com>
 *  
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; version 2 of the License ONLY.
 * 
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 * 
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

use super::*;

type CityNum = usize;
type CityWeight = f32;
type CityXYCoord = f32;

pub fn solve(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<CityWeight>>, save_run_prefix: Option<String>) -> Vec<usize> {
  let mut ordered_visits = compute_largest_triangle(node_coordinates, weights);

  while ordered_visits.len() < weights.len() {
    ordered_visits = next_step(&ordered_visits, &node_coordinates, &weights, &save_run_prefix);
  }

  // Store solution
  match &save_run_prefix {
    Some(prefix) => {
      save_state_image(format!("{}/jalgo-{:03}.png", prefix, ordered_visits.len()), &ordered_visits, &node_coordinates);
      fs::write(
        format!("{}/jalgo-path.txt", prefix),
        format!("{:?}\nDistance:{}", ordered_visits, compute_dist(weights, &ordered_visits))
      ).expect("Unable to write file");
    }
    None => { }
  }
  
  return ordered_visits;
}

// diagnostic which assumes a hamiltonian cycle of 3+ elements passed in, picks next from node_coordinates and inserts it
pub fn next_step(ordered_visits: &Vec<CityNum>, node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<CityWeight>>, save_run_prefix: &Option<String>) -> Vec<usize> {
  
  if let Some(p) = save_run_prefix { println!("============ {:?} ===============", &p); }

  let mut ordered_visits: Vec<CityNum> = ordered_visits.clone();

  let mut citynum_to_insert = 0;
  'outer: for p in 0..weights.len() {
    for ordered in &ordered_visits {
      if p == *ordered {
        continue 'outer;
      }
    }
    citynum_to_insert = p;
    break;
  }
  let citynum_to_insert = citynum_to_insert;

  let mut ideal_insert_dist_delta: CityWeight = f32::INFINITY;
  let mut ins_idx0 = 0; // 0 indicates a split of the edge that runs between 0 -> 1
  
  for from_i in 0..ordered_visits.len() {
    let to_i = (from_i+1) % ordered_visits.len();
    let from_elm = ordered_visits[from_i];
    let to_elm = ordered_visits[to_i];
    
    let this_dist_delta: CityWeight = 
      (-weights[from_elm][to_elm]) +    // removed edge counts negative
      weights[from_elm][citynum_to_insert] + // add edge from -> new
      weights[citynum_to_insert][to_elm];    // add edge new -> end
    
    if this_dist_delta < ideal_insert_dist_delta {
      ideal_insert_dist_delta = this_dist_delta;
      ins_idx0 = from_i;
    }
  }

  // .insert() takes the final resting place of the added element so
  // we keep the far end of the edge we are breaking and insert there.
  let ins_idx1 = (ins_idx0+1) % ordered_visits.len();

  ordered_visits.insert(ins_idx1, citynum_to_insert);
  // ins_idx1 points to the newly inserted city

  // ins_idx2 points to the city to the right of the newly inserted one
  let ins_idx2 = (ins_idx1+1) % ordered_visits.len();

  // // We still need to check interior edges and see
  // // if alternate routes are better. This is going to end up
  // // being a recursive step but for reasearch we may do the first 2-3 iterations manually
  // // to find the pattern.

  // for a in 0..ordered_visits.len() {
  //   if a == ins_idx1 {
  //     continue; // we do not consider the inserted city for inner edges
  //   }
  //   for b in 0..ordered_visits.len() {
  //     if b == ins_idx1 {
  //       continue; // we do not consider the inserted city for inner edges
  //     }
  //     if a == b || a == (b+1) % ordered_visits.len() || (a+1) % ordered_visits.len() == b {
  //       continue; // we do not consider edges already in the path
  //     }
  //     if b > a {
  //       continue; // we only consider half the inner edges. We choose those where a < b.
  //     }
  //     // Now we have an edge from a->b which is not in
  //     // the graph or connected to the new point.
  //     // For each of these we compute a delta to see what would
  //     // happen if we added it and removed the 3rd path on a and b.
  //     if save_run_prefix.is_some() {
  //       println!("a={} b={} len={}", a, b, ordered_visits.len());
  //     }      
  //   }
  // }


  // Research shows we may be able to correct remaining deficiencies
  // by searching for a swap and performing a single swap after each insertion.
  // Doing this 2x reduces errors further.
  for _ in 0..2 {
    for i in 0..ordered_visits.len() {
      // i is the index under consideration.
      let a = (i+(ordered_visits.len()-1)) % ordered_visits.len();
      let b = i;
      let c = (i+1) % ordered_visits.len();
      let d = (i+2) % ordered_visits.len();

      // Compute original len a->b->c->d
      let curr_len =
        weights[ordered_visits[a]][ordered_visits[b]]+
        weights[ordered_visits[b]][ordered_visits[c]]+
        weights[ordered_visits[c]][ordered_visits[d]];


      // compute len a->c->b->d
      let swapped_len =
        weights[ordered_visits[a]][ordered_visits[c]]+
        weights[ordered_visits[c]][ordered_visits[b]]+
        weights[ordered_visits[b]][ordered_visits[d]];

      if swapped_len < curr_len {
        // swap b and c
        let t = ordered_visits[c];
        ordered_visits[c] = ordered_visits[b];
        ordered_visits[b] = t;
      }

    }
  }



  // Store solution
  match &save_run_prefix {
    Some(prefix) => {
      if let Err(_e) = create_dir(prefix) {
        // Unhandled error case
      }
      save_state_image(format!("{}/jalgo-{:03}.png", prefix, ordered_visits.len()), &ordered_visits, &node_coordinates);
      fs::write(
        format!("{}/jalgo-path.txt", prefix),
        format!("{:?}\nDistance:{}", ordered_visits, compute_dist(weights, &ordered_visits))
      ).expect("Unable to write file");
    }
    None => { }
  }
  
  return ordered_visits;
}


fn compute_largest_triangle(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<f32>>) -> Vec<usize> {
  let mut ordered_visits: Vec<usize> = vec![0, 1, 2]; // holds the path as a vector of indexes relating to the city number beginning at 0

  // Make the first 2 points the furthest away in the entire graph
  for r in 0..weights.len() {
    for c in 0..weights.len() {
      if r == c { continue; }
      let best_largest_w = weights[ordered_visits[0]][ordered_visits[1]];
      let this_largest_w    = weights[r][c];
      if this_largest_w > best_largest_w {
        ordered_visits[0] = r;
        ordered_visits[1] = c;
      }
    }
  }

  // Ensure ordered_visits[2] != ordered_visits[0] or ordered_visits[1]
  while ordered_visits[2] == ordered_visits[0] || ordered_visits[2] == ordered_visits[1] {
    ordered_visits[2] = (ordered_visits[2]+1) % weights.len();
  }

  // Given the longest edge, find 
  // weight(0, 2) + weight(1, 2) (weights of both edges going to "2")
  let mut current_longest_point_len = weights[ordered_visits[0]][ordered_visits[2]] + weights[ordered_visits[1]][ordered_visits[2]];
  for r in 0..weights.len() {
    if r == ordered_visits[0] || r == ordered_visits[1] { continue; }
    let this_len = weights[ordered_visits[0]][r] + weights[ordered_visits[1]][r];
    if this_len > current_longest_point_len {
      ordered_visits[2] = r;
      current_longest_point_len = this_len;
    }
  }

  return ordered_visits;
}


// Mutates path between from_i and to_i inclusive, reversing the items between from_i and to_i.
fn reverse_slice(path: &mut Vec<usize>, from_i: usize, to_i: usize) {
  //let p_len = path.len();
  if from_i < to_i {
    // Simple case; reverse w/o overlapping
    let len = to_i - from_i;
    // Go from from_i to (from_i+len/2) swapping i and to_i-(from_i-i) at each step
    for i in from_i..(from_i+(len/2)) {
      let j = to_i-(from_i-i);
      path.swap(i, j);
    }
    // If we had a len < 2 we need to manually swap the 1 pair
    if from_i == (from_i+(len/2)) {
      path.swap(from_i, to_i);
    }
  }
  else {
    // We must wrap around the list...
    let len = (path.len() - from_i) + to_i;
    // Go from to_i to (to_i+len/2) swapping i and from_i-(to_i-i) at each step
    for n in 0..len/2 {
      let i = (from_i + n) % path.len();
      let j = (to_i + path.len() - n) % path.len();
      //println!("i={} j={} len={}", i, j, len);
      path.swap(i, j);
    }
    // If we had a len < 2 we need to manually swap the 1 pair
    if 0 == len/2 {
      path.swap(from_i, to_i);
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_slice() {
        let mut data: Vec<usize> = vec![1,2,3,4,5,6];
        reverse_slice(&mut data, 0, 2);
        assert_eq!(data, vec![3,2,1,4,5,6]);
        reverse_slice(&mut data, 0, 2);
        assert_eq!(data, vec![1,2,3,4,5,6]);

        reverse_slice(&mut data, 2, 0); // from 2 -> 0; this should wrap around
        assert_eq!(data, vec![3,2,1,6,5,4]);
        reverse_slice(&mut data, 2, 0);
        assert_eq!(data, vec![1,2,3,4,5,6]);

        reverse_slice(&mut data, 0, 1);
        assert_eq!(data, vec![2,1,3,4,5,6]);

    }
}
