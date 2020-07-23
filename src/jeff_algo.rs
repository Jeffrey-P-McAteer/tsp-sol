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
  let mut path_idx = 0; // 0 indicates a split of the edge that runs between 0 -> 1
  
  for from_i in 0..ordered_visits.len() {
    let to_i = (from_i+1) % ordered_visits.len();
    let from_elm = ordered_visits[from_i];
    let to_elm = ordered_visits[to_i];
    
    let this_dist_delta: CityWeight = 
      (-weights[from_elm][to_elm]) +    // removed edge counts negative
      weights[from_elm][citynum_to_insert] + // add edge from -> new
      weights[citynum_to_insert][to_elm];    // add edge new -> end
    
    //println!("from_elm={} to_elm={} this_dist_delta={} ideal_insert_dist_delta={}", from_elm, to_elm, this_dist_delta, ideal_insert_dist_delta);
    if this_dist_delta < ideal_insert_dist_delta {
      //println!("We are putting it between positions {} and {}", from_elm, to_elm);
      ideal_insert_dist_delta = this_dist_delta;
      path_idx = from_i;
    }
  }

  // path_idx is where we think the value citynum_to_insert should be added,
  // however we know this approach does not uphold the hamiltonian invariant.
  // Because of this we compute some properties of the graph and use them
  // to pick a different strategy. This is somewhere between a heuristic and
  // a mathematical model.

  let path_idx_plus1 = (path_idx+1) % ordered_visits.len();

  let opposite_edge0 = ( path_idx_plus1 + (ordered_visits.len()/2)) % ordered_visits.len();
  let opposite_edge1 = ( opposite_edge0+ordered_visits.len()-1 ) % ordered_visits.len();


  let len_simple = 
    weights[ ordered_visits[ path_idx ]       ][ citynum_to_insert ]+
    weights[ citynum_to_insert                ][ ordered_visits[ path_idx_plus1 ] ]+
    weights[ ordered_visits[ opposite_edge0 ] ][ ordered_visits[ opposite_edge1 ] ];

  let len_inverted_a = 
    weights[ ordered_visits[ path_idx ]       ][ citynum_to_insert ]+
    weights[ citynum_to_insert                ][ ordered_visits[ opposite_edge1 ] ]+
    weights[ ordered_visits[ path_idx_plus1 ] ][ ordered_visits[ opposite_edge0 ] ];

  let len_inverted_b =  // This is still a work in progress 07/17
    weights[ ordered_visits[ path_idx_plus1 ]       ][ citynum_to_insert ]+
    weights[ citynum_to_insert                ][ ordered_visits[ opposite_edge0 ] ]+
    weights[ ordered_visits[ path_idx ] ][ ordered_visits[ opposite_edge1 ] ];

  if save_run_prefix.is_some() { println!("============ {:?} ===============", &save_run_prefix); }
  if save_run_prefix.is_some() { println!("path_idx={} path_idx_plus1={} opposite_edge0={} opposite_edge1={}", path_idx, path_idx_plus1, opposite_edge0, opposite_edge1); }
  if save_run_prefix.is_some() { println!("len_simple={} len_inverted_a={} len_inverted_b={}", len_simple, len_inverted_a, len_inverted_b); }

  if len_simple <= len_inverted_a && len_simple <= len_inverted_b {
    if save_run_prefix.is_some() { println!("simple insert, ordered_visits={:?}", &ordered_visits); }
    // Do the simple insert. This is the correct move for ~80% of graphs
    ordered_visits.insert(path_idx_plus1, citynum_to_insert);

  }
  else if len_inverted_a <= len_simple && len_inverted_a <= len_inverted_b  {
    if save_run_prefix.is_some() { println!("len_inverted_a, ordered_visits={:?}", &ordered_visits); }
    // With our test city this covers quadrants 4 and 2
    reverse_slice(&mut ordered_visits, path_idx_plus1, opposite_edge1);
    ordered_visits.insert(path_idx_plus1, citynum_to_insert);

  }
  else if len_inverted_b <= len_simple && len_inverted_b <= len_inverted_a  {
    if save_run_prefix.is_some() { println!("len_inverted_b, ordered_visits={:?}", &ordered_visits); }
    // With our test city this covers quadrants 1 and 3
    reverse_slice(&mut ordered_visits, path_idx_plus1, opposite_edge1);
    ordered_visits.insert(opposite_edge0, citynum_to_insert);

  }
  else {
    panic!("This case should be impossible.");
  }


  // Now we handle edge cases where we can fix them by swapping N and N+1
  let ov_len = ordered_visits.len();
  for from_i in 0..ov_len {
    let a = (from_i + ov_len - 3) % ov_len;
    let b = (from_i + ov_len - 2) % ov_len; // Considered for swap
    let c = (from_i + ov_len - 1) % ov_len; // Considered for swap
    let d = (from_i + ov_len - 0) % ov_len;

    let orig_len = 
      weights[ ordered_visits[ a ] ][ ordered_visits[ b ] ]+
      weights[ ordered_visits[ b ] ][ ordered_visits[ c ] ]+
      weights[ ordered_visits[ c ] ][ ordered_visits[ d ] ];

    let swap_len = 
      weights[ ordered_visits[ a ] ][ ordered_visits[ c ] ]+
      weights[ ordered_visits[ c ] ][ ordered_visits[ b ] ]+
      weights[ ordered_visits[ b ] ][ ordered_visits[ d ] ];

    if save_run_prefix.is_some() { println!("before swap from_i={} ordered_visits={:?}", from_i, ordered_visits); }
    if swap_len < orig_len {
      if save_run_prefix.is_some() { println!("Swapping idx:{} val:{} and idx:{} val:{} because {} < {}", b, ordered_visits[b], c, ordered_visits[c], swap_len, orig_len); }
      ordered_visits.swap(b, c); // values at b and c are swapped
      if save_run_prefix.is_some() { println!("after swap ordered_visits={:?}", ordered_visits); }
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
