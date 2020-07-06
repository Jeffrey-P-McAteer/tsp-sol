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

pub fn solve(node_coordinates: &Vec<(usize, f32, f32)>, weights: &Vec<Vec<f32>>, save_run_prefix: Option<String>) -> Vec<usize> {
  // We begin with points 0, 1, and 2.
  // These will be overwritten in the largest-triangle-fining process
  let mut ordered_visits: Vec<usize> = vec![0, 1, 2]; // holds the path as a vector of indexes relating to the city number beginning at 0
  
  match &save_run_prefix {
    Some(prefix) => { if let Err(_e) = create_dir(prefix) { } }
    None => { }
  }
  
  // If we have 3 or fewer points, we're done. min bound is O(1), good job folks.
  if weights.len() <= 3 {
    return (&ordered_visits[0..weights.len()]).to_vec();
  }
  
  { // Find largest triangle
    { // Make the first 2 points the furthest away in the entire graph
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
    }
    { // Ensure ordered_visits[2] != ordered_visits[0] or ordered_visits[1]
      while ordered_visits[2] == ordered_visits[0] || ordered_visits[2] == ordered_visits[1] {
        ordered_visits[2] = (ordered_visits[2]+1) % weights.len();
      }
    }
    //save_state_image(format!("./views/0-1.png"), &ordered_visits, &node_coordinates, &(0.0, 0.0));
    { // Given the longest edge, find 
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
    }
  }
  // Holds all points not in ordered_visits
  let mut unordered_visits: Vec<usize> = Vec::with_capacity(weights.len()-3);
  'outer: for p in 0..weights.len() {
    for ordered in &ordered_visits {
      if p == *ordered {
        continue 'outer;
      }
    }
    // we haven't continued, therefore we are not in odered_visits
    unordered_visits.push(p);
  }
  
  while ordered_visits.len() < weights.len() {

    ordered_visits = next_step(&ordered_visits, &node_coordinates, &weights, &save_run_prefix);

    // let (furthest_non_collected_point_i,
    //      ordered_idx,
    //      unordered_idx) = compute_furthest(&ordered_visits, &unordered_visits, &weights, &node_coordinates);
    
    // match &save_run_prefix {
    //   Some(prefix) => { save_state_image(format!("{}/jalgo-{:03}.png", prefix, ordered_visits.len()), &ordered_visits, &node_coordinates); }
    //   None => { }
    // }

    // unordered_visits.remove(unordered_idx);
    
    // let ordered_idx = (ordered_idx+1) % ordered_visits.len();
    // ordered_visits.insert(ordered_idx, furthest_non_collected_point_i);
    
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
pub fn next_step(ordered_visits: &Vec<usize>, node_coordinates: &Vec<(usize, f32, f32)>, weights: &Vec<Vec<f32>>, save_run_prefix: &Option<String>) -> Vec<usize> {

  let mut ordered_visits: Vec<usize> = ordered_visits.clone();

  match &save_run_prefix {
    Some(prefix) => { if let Err(_e) = create_dir(prefix) { } }
    None => { }
  }

  // Holds all points not in ordered_visits
  let mut unordered_visits: Vec<usize> = Vec::with_capacity(weights.len()-3);
  'outer: for p in 0..weights.len() {
    for ordered in &ordered_visits {
      if p == *ordered {
        continue 'outer;
      }
    }
    // we haven't continued, therefore we are not in odered_visits
    unordered_visits.push(p);
  }

  // let (furthest_non_collected_point_i,
  //      ordered_idx,
  //      unordered_idx) = compute_furthest(&ordered_visits, &unordered_visits, &weights, &node_coordinates);
  

  let unordered_idx = 0;
  let furthest_i = unordered_visits[unordered_idx];
  
  // Let's re-scope some variables to be immutable now that we've calculated them
  let furthest_i = furthest_i; // idx to weight matrix
  let unordered_idx = unordered_idx;
  // println!("furthest_i={}", furthest_i);
  // Now determine shortest split & merge, set path_idx=
  let mut ideal_insert_dist_delta: f32 = f32::INFINITY;
  let mut path_idx = 0; // 0 indicates a split of the edge that runs between 0 -> 1
  
  for from_i in 0..ordered_visits.len() {
    let to_i = (from_i+1) % ordered_visits.len();
    let from_elm = ordered_visits[from_i];
    let to_elm = ordered_visits[to_i];
    
    let this_dist_delta: f32 = 
      (-weights[from_elm][to_elm]) +    // removed edge counts negative
      weights[from_elm][furthest_i] + // add edge from -> new
      weights[furthest_i][to_elm];    // add edge new -> end
    
    //println!("from_elm={} to_elm={} this_dist_delta={} ideal_insert_dist_delta={}", from_elm, to_elm, this_dist_delta, ideal_insert_dist_delta);
    if this_dist_delta < ideal_insert_dist_delta {
      //println!("We are putting it between positions {} and {}", from_elm, to_elm);
      ideal_insert_dist_delta = this_dist_delta;
      path_idx = from_i;
    }
  }
  
  let furthest_non_collected_point_i = furthest_i/*idx to weight matrix*/;
  let ordered_idx = path_idx;
  let unordered_idx = unordered_idx;

  match &save_run_prefix {
    Some(prefix) => { save_state_image(format!("{}/jalgo-{:03}.png", prefix, ordered_visits.len()), &ordered_visits, &node_coordinates); }
    None => { }
  }

  unordered_visits.remove(unordered_idx);

  // If the current ordered_visits is even (2, 4, 6 etc length)
  // we select the edge opposite ordered_idx -> (ordered_idx+1) % ordered_visits.len()
  // and remove it as well.
  if ordered_visits.len() % 2 == 0 {
    let opposite_edge_i0 = (ordered_idx + (ordered_visits.len()/2)) % ordered_visits.len();
    let opposite_edge_i1 = (ordered_idx + 1 + (ordered_visits.len()/2)) % ordered_visits.len();

    // Now we check both combos to catch a one-off case 
    let mut len_pt1 = 0.0;
    for i in 0..(ordered_idx-1) {
      len_pt1 += weights[ordered_visits[i]][ordered_visits[i+1]];
    }

    let mut len_pt2 = 0.0;
    for i in ordered_idx.. {
      len_pt2 += weights[ordered_visits[i]][ordered_visits[i+1]];
    }

    std::unimplemented!();

    // check PT1..ordered_idx -> furthest_non_collected_point_i -> (ordered_idx+1)..PT2

    // check opposite_edge_i0..ordered_idx -> furthest_non_collected_point_i -> (ordered_idx+1)..opposite_edge_i1


  }
  else {
    // Simplest insertion when len() == odd
    let ordered_idx = (ordered_idx+1) % ordered_visits.len();
    ordered_visits.insert(ordered_idx, furthest_non_collected_point_i);
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

fn compute_furthest(path: &Vec<usize>, unordered: &Vec<usize>, weights: &Vec<Vec<f32>>, _locations: &Vec<(usize, f32, f32)>)
  ->
  (usize /*point i*/, usize /*points idx in path*/, usize /*points idx in unordered*/)
{
  let unordered_idx = 0;
  let furthest_i = unordered[unordered_idx];
  
  // Let's re-scope some variables to be immutable now that we've calculated them
  let furthest_i = furthest_i; // idx to weight matrix
  let unordered_idx = unordered_idx;
  // println!("furthest_i={}", furthest_i);
  // Now determine shortest split & merge, set path_idx=
  let mut ideal_insert_dist_delta: f32 = f32::INFINITY;
  let mut path_idx = 0; // 0 indicates a split of the edge that runs between 0 -> 1
  
  for from_i in 0..path.len() {
    let to_i = (from_i+1) % path.len();
    let from_elm = path[from_i];
    let to_elm = path[to_i];
    
    let this_dist_delta: f32 = 
      (-weights[from_elm][to_elm]) +    // removed edge counts negative
      weights[from_elm][furthest_i] + // add edge from -> new
      weights[furthest_i][to_elm];    // add edge new -> end
    
    //println!("from_elm={} to_elm={} this_dist_delta={} ideal_insert_dist_delta={}", from_elm, to_elm, this_dist_delta, ideal_insert_dist_delta);
    if this_dist_delta < ideal_insert_dist_delta {
      //println!("We are putting it between positions {} and {}", from_elm, to_elm);
      ideal_insert_dist_delta = this_dist_delta;
      path_idx = from_i;
    }
  }
  
  return (furthest_i/*idx to weight matrix*/, path_idx, unordered_idx);
}
