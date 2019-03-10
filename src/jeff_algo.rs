
use super::*;

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
  // Compute triangle center.
  // We update this on every point insertion to the ideal path.
  let mut center = compute_center(&ordered_visits, &node_coordinates);
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
  
  // duplicate starting data N times to make our racers;
  // every step we will compare their lengths and if one is smaller we keep that one
  let racers = 3;
  // if a racer does worse then the others for this many consecurive turns,
  // we overwrite with the best one.
  let losses_before_overwritten = 5;
  
  let mut race_ordered_visits: Vec<Vec<usize>> = vec![];
  let mut race_unordered_visits: Vec<Vec<usize>> = vec![];
  let mut race_loss_counters: Vec<usize> = vec![];
  
  for _ in 0..racers {
    race_ordered_visits.push(
      ordered_visits.clone()
    );
    race_unordered_visits.push(
      unordered_visits.clone()
    );
    race_loss_counters.push(
      0
    );
  }
  
  // re-bind to prevent using these later
  #[allow(unused_variables)]
  let ordered_visits: isize = 5;
  #[allow(unused_variables)]
  let unordered_visits: isize = 5;
  
  // Used when we return
  let mut best_racer_i = 0;
  
  while race_ordered_visits[best_racer_i].len() < weights.len() {
    // Tick each racer forward, with some difference `i`
    for i in 0..racers {
      
      let (furthest_non_collected_point_i,
           ordered_idx,
           unordered_idx) =
        if i < 1      { compute_furthest(&race_ordered_visits[i], &race_unordered_visits[i], &weights, &node_coordinates, &center) }
        else if i < 2 { compute_furthest_min_1(&race_ordered_visits[i], &race_unordered_visits[i], &weights, &node_coordinates, &center) }
        else          { compute_furthest_min_2(&race_ordered_visits[i], &race_unordered_visits[i], &weights, &node_coordinates, &center) };
      
      match &save_run_prefix {
        Some(prefix) => {
          save_state_image(format!("{}/jalgo-r{}-{:03}.png", prefix, i, race_ordered_visits[i].len()), &race_ordered_visits[i], &node_coordinates, &center);
        }
        None => { }
      }
      
      if race_ordered_visits[i].contains(&furthest_non_collected_point_i) {
        panic!("furthest_non_collected_point_i has been collected in race_ordered_visits[i] already!")
      }
      
      if race_unordered_visits[i][unordered_idx] == furthest_non_collected_point_i {
        race_unordered_visits[i].remove(unordered_idx);
      }
      else {
        // Look for correct index
        race_unordered_visits[i].retain(|&x| x != furthest_non_collected_point_i);
      }
      
      let ordered_idx = (ordered_idx+1) % race_ordered_visits[i].len();
      race_ordered_visits[i].insert(ordered_idx, furthest_non_collected_point_i);
      
      center = compute_center(&race_ordered_visits[i], &node_coordinates);
      
    }
    
    // Compare racer total
    let mut best_racer_len = f32::MAX;
    let mut overwrite_queue: Vec<usize> = vec![];
    let common_racer_points = race_ordered_visits[0].len();
    
    for i in 0..racers {
      if race_ordered_visits[i].len() != common_racer_points {
        panic!("BIG PROBLEM: i and common_racer_points lengths do not match!");
      }
      let racer_len = compute_dist(weights, &race_ordered_visits[i]);
      if racer_len < best_racer_len {
        race_loss_counters[i] = 0;
        best_racer_i = i;
        best_racer_len = racer_len;
      }
      else {
        // We lost, increase loss counter
        race_loss_counters[i] += 1;
        if race_loss_counters[i] > losses_before_overwritten {
          // Queue overwriting this racer with whatever is determined to be best at end of loop
          overwrite_queue.push(i);
        }
      }
    }
    
    // Overwrite underperforming racers
    for i in 0..overwrite_queue.len() {
      race_loss_counters[i] = 0;
      // never happens
      if race_ordered_visits[i].len() != race_ordered_visits[best_racer_i].len() {
        panic!("BIG PROBLEM: failing i and best_racer_i lengths do not match!");
      }
      race_ordered_visits[i] = race_ordered_visits[best_racer_i].clone();
      race_unordered_visits[i] = race_unordered_visits[best_racer_i].clone();
    }
    
  }
  
  // Store solution(s)
  for i in 0..racers {
    match &save_run_prefix {
      Some(prefix) => {
        save_state_image(format!("{}/jalgo-r{}-{:03}.png", prefix, i, race_ordered_visits[i].len()), &race_ordered_visits[i], &node_coordinates, &center);
        fs::write(
          format!("{}/jalgo-r{}-path.txt", prefix, i),
          format!("Best:{}\n{:?}\nDistance:{}", if i == best_racer_i { "true" } else { "false" }, race_ordered_visits[i], compute_dist(weights, &race_ordered_visits[i]))
        ).expect("Unable to write file");
      }
      None => { }
    }
  }
  
  return race_ordered_visits[best_racer_i].clone();
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


fn compute_furthest_min_1(path: &Vec<usize>, unordered: &Vec<usize>, weights: &Vec<Vec<f32>>, locations: &Vec<(usize, f32, f32)>, center: &(f32, f32))
  ->
  (usize /*point i*/, usize /*points idx in path*/, usize /*points idx in unordered*/)
{
  //println!("path = {:?}", path);
  //println!("unordered = {:?}", unordered);
  //println!("!!weights.len() = {}", weights.len());
  let (furthest_non_collected_point_i,
           ordered_idx,
           unordered_idx) = compute_furthest(path, unordered, weights, locations, center);
  //println!("furthest_non_collected_point_i={}", furthest_non_collected_point_i);
  if unordered.len() <= 1 { // removing 1 element gives us 0
    return (furthest_non_collected_point_i,
           ordered_idx,
           unordered_idx);
  }
  else {
    let mut unordered_clone: Vec<usize> = (*unordered).clone();
    // Remove the best one from consideration
    unordered_clone.remove(unordered_idx);
    
    return compute_furthest(path, &unordered_clone, weights, locations, center);
  }
}

fn compute_furthest_min_2(path: &Vec<usize>, unordered: &Vec<usize>, weights: &Vec<Vec<f32>>, locations: &Vec<(usize, f32, f32)>, center: &(f32, f32))
  ->
  (usize /*point i*/, usize /*points idx in path*/, usize /*points idx in unordered*/)
{
  let (furthest_non_collected_point_i,
           ordered_idx,
           unordered_idx) = compute_furthest(path, unordered, weights, locations, center);
  
  if unordered.len() < 3 {
    return (furthest_non_collected_point_i,
           ordered_idx,
           unordered_idx);
  }
  else {
    let mut unordered_clone = unordered.clone();
    // Remove the best one
    unordered_clone.remove(unordered_idx);
    
    return compute_furthest_min_1(path, &unordered_clone, weights, locations, center);
  }
}
