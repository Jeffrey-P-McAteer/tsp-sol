
use super::*;

use permutohedron::LexicalPermutation;

pub fn solve(node_coordinates: &Vec<(usize, f32, f32)>, weights: &Vec<Vec<f32>>, save_run_prefix: Option<String>) -> Vec<usize> {
  match &save_run_prefix {
    Some(prefix) => {
      create_dir(prefix).expect("Could not create directory");
    }
    None => { }
  }
  
  let mut current_path = vec![];
  for i in 0..weights.len() {
    current_path.push(i);
  }
  let mut best_path = current_path.clone();
  let mut best_path_dist = compute_dist(weights, &best_path);
  
  loop {
    //println!("current_path = {:?}", current_path);
    let this_dist = compute_dist(weights, &current_path);
    if this_dist < best_path_dist {
      best_path = current_path.clone();
      best_path_dist = this_dist;
    }
    
    if !current_path.next_permutation() {
      break;
    }
  }
  
  // Store solution
  match &save_run_prefix {
    Some(prefix) => {
      save_state_image(format!("{}/brute-{:03}.png", prefix, best_path.len()), &best_path, &node_coordinates, &compute_center(&best_path, &node_coordinates));
      fs::write(
        format!("{}/brute-path.txt", prefix),
        format!("{:?}\nDistance:{}", best_path, compute_dist(weights, &best_path))
      ).expect("Unable to write file");
      fs::write(
        format!("{}/node-coordinates.txt", prefix),
        format!("{:?}", node_coordinates)
      ).expect("Unable to write file");
    }
    None => { }
  }
  
  return best_path;
}

