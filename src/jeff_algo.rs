
use super::*;

use std::cmp::Ordering;

#[derive(Clone)]
struct SolutionTracker {
  pub ordered_visits: Vec<usize>,
  pub unordered_visits: Vec<usize>,
  pub center: (f32, f32),
}

impl SolutionTracker {
  // Mutates object
  fn step_forward_self(&mut self, ideal_point_to_select_offset: usize, ideal_insertion_idx_offset: usize) {
    
    std::unimplemented!()
    
  }
  // Performs a step, then clones split_n copies of the stepped forward
  pub fn step_forward(&mut self, split_n: usize, ideal_point_to_select_offset: usize, ideal_insertion_idx_offset: usize) -> Vec<SolutionTracker> {
    self.step_forward_self(ideal_point_to_select_offset, ideal_insertion_idx_offset);
    let v = vec![];
    for _ in 0..split_n {
      v.push(self.clone());
    }
    return v;
  }
  
}

pub fn solve(node_coordinates: &Vec<(usize, f32, f32)>, weights: &Vec<Vec<f32>>, save_run_prefix: Option<String>) -> Vec<usize> {
  // We begin with points 0, 1, and 2.
  // These will be overwritten in the largest-triangle-fining process
  let mut ordered_visits: Vec<usize> = vec![0, 1, 2]; // holds the path as a vector of indexes relating to the city number beginning at 0
  
  match &save_run_prefix {
    Some(prefix) => {
      if ! Path::new(prefix).exists() {
        create_dir(prefix).expect("Could not create directory");
      }
    }
    None => { }
  }
  
  // If we have 3 or fewer points, we're done. min bound is O(1), good job folks.
  if weights.len() <= 3 {
    return ordered_visits;
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
  
  // How many times do we split on each node going down?
  let SPLIT_N = 2;
  // How many iterations before a cull operation?
  let DEPTH_N = 3;
  // How many ideal do we keep during a cull?
  let KEPT_N = 2;
  
  let mut sol_set: Vec<SolutionTracker> = vec![];
  
  // Insert beginning SolutionTracker
  sol_set.push(
    SolutionTracker {
      ordered_visits: ordered_visits,
      unordered_visits: unordered_visits,
      center: compute_center(&ordered_visits, &node_coordinates),
    }
  );
  
  // re-bind to prevent using these later
  #[allow(unused_variables)]
  let ordered_visits: isize = 5;
  #[allow(unused_variables)]
  let unordered_visits: isize = 5;
  #[allow(unused_variables)]
  let center: isize = 5;
  
  // Used when we return
  let mut best_racer_i = 0;
  
  let mut outer_i = 1; // used to cull every DEPTH_N iterations
  
  while sol_set[0].ordered_visits.len() < weights.len() {
    let mut new_sol_set = vec![];
    for i in 0..sol_set.len() {
      new_sol_set.extend(
        sol_set[i].step_forward()
      );
    }
    
    // Write new row into sol_set, replacing contents
    sol_set.clear();
    sol_set.extend(new_sol_set);
    
    if outer_i % DEPTH_N == 0 {
      // Perform cull to remove all except the best KEPT_N
      
      sol_set.sort_by(|a, b| {
        let a_d: f32 = compute_dist(weights, &a.ordered_visits);
        let b_d: f32 = compute_dist(weights, &b.ordered_visits);
        if a_d < b_d { Ordering::Less }
        else if a_d > b_d { Ordering::Greater }
        else { Ordering::Equal }
      });
      // Best are at [0], [1], etc...
      sol_set.truncate(KEPT_N);
      
    }
    
  }
  
  // Store solution(s)
  for i in 0..sol_set.len() {
    match &save_run_prefix {
      Some(prefix) => {
        save_state_image(format!("{}/jalgo-r{}-{:03}.png", prefix, i, sol_set[i].ordered_visits.len()), &sol_set[i].ordered_visits, &node_coordinates, &sol_set[i].center);
        fs::write(
          format!("{}/jalgo-r{}-path.txt", prefix, i),
          format!("Best:{}\n{:?}\nDistance:{}", if i == best_racer_i { "true" } else { "false" }, sol_set[i].ordered_visits, compute_dist(weights, &sol_set[i].ordered_visits))
        ).expect("Unable to write file");
      }
      None => { }
    }
  }
  
  // Last sort
  sol_set.sort_by(|a, b| {
    let a_d: f32 = compute_dist(weights, &a.ordered_visits);
    let b_d: f32 = compute_dist(weights, &b.ordered_visits);
    if a_d < b_d { Ordering::Less }
    else if a_d > b_d { Ordering::Greater }
    else { Ordering::Equal }
  });
  
  // Return shortest
  return sol_set[0].ordered_visits.clone();
}

fn compute_furthest(path: &Vec<usize>, unordered: &Vec<usize>, weights: &Vec<Vec<f32>>, locations: &Vec<(usize, f32, f32)>, center: &(f32, f32))
  ->
  (usize /*point i*/, usize /*points idx in path*/, usize /*points idx in unordered*/)
{
  let mut unordered_idx = 0;
  let mut furthest_i = unordered[unordered_idx];
  let mut furthest_i_dist_from_center: f32 = -100.0;
  
  // for every point not in the solution...
  for i in 0..unordered.len() {
    let unord_elm = unordered[i];
    let this_dist_from_center: f32 = (
      (center.0/*x*/ - locations[unord_elm].1/*x*/).powf(2.0) +
      (center.1/*y*/ - locations[unord_elm].2/*y*/).powf(2.0)
    ).sqrt();
    if this_dist_from_center > furthest_i_dist_from_center {
      furthest_i_dist_from_center = this_dist_from_center;
      furthest_i = unord_elm;
      unordered_idx = i;
      // we don't know where it is GOING yet.
    }
  }
  
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

fn compute_furthest_min_x(path: &Vec<usize>, unordered: &Vec<usize>, weights: &Vec<Vec<f32>>, locations: &Vec<(usize, f32, f32)>, center: &(f32, f32), x: usize)
  ->
  (usize /*point i*/, usize /*points idx in path*/, usize /*points idx in unordered*/)
{
  let (furthest_non_collected_point_i,
           ordered_idx,
           unordered_idx) = compute_furthest(path, unordered, weights, locations, center);

  if x < 1 || unordered.len() <= 1 {
    // We'll run out if we continue
    return (furthest_non_collected_point_i,
           ordered_idx,
           unordered_idx);
  }
  else {
    let mut unordered_clone: Vec<usize> = (*unordered).clone();
    // Remove the best one from consideration
    unordered_clone.remove(unordered_idx);
    
    return compute_furthest_min_x(path, &unordered_clone, weights, locations, center, x-1);
  }
}

fn number_duplicates(path: &Vec<usize>, elm: usize) -> usize {
  let mut count = 0;
  for p in path.iter() {
    if p == &elm {
      count += 1;
    }
  }
  return count;
}
