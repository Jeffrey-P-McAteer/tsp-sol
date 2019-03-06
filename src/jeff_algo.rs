
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
  
  while ordered_visits.len() < weights.len() {
    let (furthest_non_collected_point_i,
         ordered_idx,
         unordered_idx) = compute_furthest(&ordered_visits, &unordered_visits, &weights, &node_coordinates, &center);
    
    match &save_run_prefix {
      Some(prefix) => {
        save_state_image(format!("{}/jalgo-{:03}.png", prefix, ordered_visits.len()), &ordered_visits, &node_coordinates, &center);
      }
      None => { }
    }
    //print_path_metadata(&ordered_visits, &weights);
    //save_state_image(format!("./views/{}.png", ordered_visits.len()), &ordered_visits, &node_coordinates, &center);
    // println!("ordered_visits = {:?}", ordered_visits);
    // println!("unordered_visits = {:?}", unordered_visits);
    
    unordered_visits.remove(unordered_idx);
    //println!("Inserting {} at urdered[{}]", furthest_non_collected_point_i, ordered_idx);
    
    let ordered_idx = (ordered_idx+1) % ordered_visits.len();
    ordered_visits.insert(ordered_idx, furthest_non_collected_point_i);
    
    center = compute_center(&ordered_visits, &node_coordinates);
    //println!(" = = = = ");
    
    // We may have made a mistake around edge-case corners
    let p_tot_len = path_total_len(&ordered_visits, weights);
    for idx in (ordered_idx+ordered_visits.len()-4)..(ordered_idx+ordered_visits.len()+4) {
      let idx = idx % ordered_visits.len();
      if compute_switchback_len(idx, 1, &ordered_visits, weights) < p_tot_len {
        println!("Switchback is improving our distance!");
        apply_switchback(idx, 1, &mut ordered_visits);
      }
    }
    // Check the same thing in the opposite directon
    // We may have made a mistake around edge-case corners
    let p_tot_len = path_total_len(&ordered_visits, weights);
    for idx in (ordered_idx+ordered_visits.len()-4)..(ordered_idx+ordered_visits.len()+4) {
      let idx = idx % ordered_visits.len();
      if compute_switchback_len(idx, -1, &ordered_visits, weights) < p_tot_len {
        println!("Switchback is improving our distance!");
        apply_switchback(idx, -1, &mut ordered_visits);
      }
    }
  }
  
  // Store solution
  match &save_run_prefix {
    Some(prefix) => {
      save_state_image(format!("{}/jalgo-{:03}.png", prefix, ordered_visits.len()), &ordered_visits, &node_coordinates, &center);
      fs::write(
        format!("{}/jalgo-path.txt", prefix),
        format!("{:?}\nDistance:{}", ordered_visits, compute_dist(weights, &ordered_visits))
      ).expect("Unable to write file");
    }
    None => { }
  }
  
  return ordered_visits;
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

fn compute_switchback_len(idx: usize, direction: isize, path: &Vec<usize>, weights: &Vec<Vec<f32>>) -> f32 {
  let mut path_clone = path.clone();
  apply_switchback(idx as usize, direction, &mut path_clone);
  return path_total_len(&path_clone, weights);
}

// Mutates path according to switchback algo
fn apply_switchback(idx: usize, direction: isize, path: &mut Vec<usize>) {
  if direction > 0 {
    // N+2 and N+1 swap positions
    let n_p_1 = (idx + 1) % path.len();
    let n_p_2 = (idx + 2) % path.len();
    path.swap(n_p_1, n_p_2);
  }
  else {
    // N-2 and N-1 swap positions
    let n_p_1 = ((idx + path.len()) - 1) % path.len();
    let n_p_2 = ((idx + path.len()) - 2) % path.len();
    path.swap(n_p_1, n_p_2);
  }
}

// A and B are values of path elements
fn edge_len(a:isize, b:isize, weights: &Vec<Vec<f32>>) -> f32 {
  let a: usize = ((a + weights.len() as isize) as usize % weights.len()) as usize;
  let b: usize = ((b + weights.len() as isize) as usize % weights.len()) as usize;
  return weights[a][b];
}

fn path_total_len(path: &Vec<usize>, weights: &Vec<Vec<f32>>) -> f32 {
  let mut total = 0.0;
  for i in 1..(path.len()+1) {
    total += edge_len(path[(i+path.len()-1) % path.len()] as isize, path[i % path.len()] as isize, weights);
  }
  return total;
}