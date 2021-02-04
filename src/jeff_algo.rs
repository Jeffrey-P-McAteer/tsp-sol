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

  // More detailed strat:
  // for all N:
  //   remove N from ordered_visits
  //   insert citynum_to_insert using insert_point_step
  //   insert N using insert_point_step
  // keep the smallest delta from these ops

  let mut strat_b_best_tour_delta = f32::INFINITY;
  let mut strat_b_best_tour_n = 0;
  let mut strat_b_best_tour_m = 0;

  for n in 0..ordered_visits.len() {
    let removed_citynum_n = ordered_visits.remove(n);

    let n_left_citynum = ordered_visits[ (n + ordered_visits.len() - 1) % ordered_visits.len() ];
    let n_right_citynum = ordered_visits[ (n) % ordered_visits.len() ];

    // Delta must begin with the removal of 2 edges above
    let mut this_delta: f32 = (-weights[n_left_citynum][removed_citynum_n]) + (-weights[removed_citynum_n][n_right_citynum]) + weights[n_left_citynum][n_right_citynum];

    for m in 0..ordered_visits.len() {

      let removed_citynum_m = ordered_visits.remove(m);

      let m_left_citynum = ordered_visits[ (m + ordered_visits.len() - 1) % ordered_visits.len() ];
      let m_right_citynum = ordered_visits[ (m) % ordered_visits.len() ];

      // Delta must begin with the removal of 2 edges above
      let mut this_delta: f32 = this_delta;
      this_delta += (-weights[m_left_citynum][removed_citynum_m]) + (-weights[removed_citynum_m][m_right_citynum]) + weights[m_left_citynum][m_right_citynum];
      this_delta += insert_point_step(&mut ordered_visits, node_coordinates, weights, citynum_to_insert);
      this_delta += insert_point_step(&mut ordered_visits, node_coordinates, weights, removed_citynum_m);
      this_delta += insert_point_step(&mut ordered_visits, node_coordinates, weights, removed_citynum_n);

      if this_delta < strat_b_best_tour_delta {
        // Keep changes, update strat_b_best_tour_delta
        strat_b_best_tour_delta = this_delta;
        strat_b_best_tour_n = n;
        strat_b_best_tour_m = m;
      }

      // Undo changes so ordered_visits is identical to the beginning
      remove_point_step(&mut ordered_visits, node_coordinates, weights, removed_citynum_n);
      remove_point_step(&mut ordered_visits, node_coordinates, weights, removed_citynum_m);
      remove_point_step(&mut ordered_visits, node_coordinates, weights, citynum_to_insert);
      ordered_visits.insert(m, removed_citynum_m);

    }

    ordered_visits.insert(n, removed_citynum_n);

  }

  // Apply strat b
  let removed_citynum_n = ordered_visits.remove(strat_b_best_tour_n);
  let removed_citynum_m = ordered_visits.remove(strat_b_best_tour_m);
  insert_point_step(&mut ordered_visits, node_coordinates, weights, citynum_to_insert);
  insert_point_step(&mut ordered_visits, node_coordinates, weights, removed_citynum_m);
  insert_point_step(&mut ordered_visits, node_coordinates, weights, removed_citynum_n);

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

// Modified args instead of returning a clone
// returns the delta from this modification (aka how much did len(ordered_visits) change, smaller is better.)
fn insert_point_step(ordered_visits: &mut Vec<CityNum>, node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<CityWeight>>, citynum_to_insert: CityNum) -> CityWeight {
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

  return ideal_insert_dist_delta;
}

// undoes insert_point_step given the same citynum_to_insert and returns the delta
fn remove_point_step(ordered_visits: &mut Vec<CityNum>, node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<CityWeight>>, citynum_to_insert: CityNum) -> CityWeight {
  let from_i = ordered_visits.iter().position(|&val| val == citynum_to_insert).unwrap();

  ordered_visits.remove(from_i); // returns citynum_to_insert

  let from_i = (from_i+ordered_visits.len()-1) % ordered_visits.len();

  let to_i = (from_i+1) % ordered_visits.len();
  let from_elm = ordered_visits[from_i];
  let to_elm = ordered_visits[to_i];

  let this_dist_delta: CityWeight = 
      weights[from_elm][to_elm] +    // Added edge counts positive
      (-weights[from_elm][citynum_to_insert]) + // removed edge from -> new
      (-weights[citynum_to_insert][to_elm]);    // removed edge new -> end

  return this_dist_delta;
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
