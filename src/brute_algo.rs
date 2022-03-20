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

use permutohedron::LexicalPermutation;

pub fn solve(node_coordinates: &Vec<(usize, fp, fp)>, weights: &Vec<Vec<fp>>, save_run_prefix: Option<String>) -> Vec<usize> {
  let mut current_path = vec![];
  for i in 0..weights.len() {
    current_path.push(i);
  }
  // We, uh... need to go backwards until we hit the "first ordered permutation".
  // This was a bug in the original implementation which apparently just went from
  // random permutation index -> last sorted permutation.
  loop {
    if !current_path.prev_permutation() {
      break;
    }
  }

  let mut best_path = current_path.clone();
  let mut best_path_dist = compute_dist(weights, &best_path);
  
  loop {
    //*
    //println!("current_path = {:?}", current_path);
    let this_dist = compute_dist(weights, &current_path);
    if this_dist < best_path_dist {
      best_path = current_path.clone();
      best_path_dist = this_dist;
    }
    
    if !current_path.next_permutation() {
      break;
    }
    /**/

    /*
    // we copy in https://docs.rs/permutohedron/0.2.4/src/permutohedron/lexical.rs.html#34
    // and only track path distance deltas for a huge performance boost.
    let mut this_dist = best_path_dist;
    
    // Step 1: Identify the longest, rightmost weakly decreasing part of the vector
    let mut i = current_path.len() - 1;
    while i > 0 && current_path[i-1] >= current_path[i] {
        i -= 1;
    }

    // If that is the entire vector, this is the last-ordered permutation.
    if i == 0 {
        break;
    }

    // Step 2: Find the rightmost element larger than the pivot (i-1)
    let mut j = current_path.len() - 1;
    while j >= i && current_path[j] <= current_path[i-1]  {
        j -= 1;
    }

    // Step 3: Swap that element with the pivot
    current_path.swap(j, i-1);

    // Step 4: Reverse the (previously) weakly decreasing part
    current_path[i..].reverse();

    // Because we've only modified a part of the distance, we have significantly fewer
    // floating point ops required to make the same decision!
    if this_dist < best_path_dist {
      best_path = current_path.clone();
      best_path_dist = this_dist;
    }
    */

  }
  
  // Store solution
  match &save_run_prefix {
    Some(prefix) => {
      save_state_image(format!("{}/brute-{:03}.png", prefix, best_path.len()), &best_path, &node_coordinates);
      fs::write(
        format!("{}/brute-path.txt", prefix),
        format!("{:?}\nDistance:{}", best_path, compute_dist(weights, &best_path))
      ).expect("Unable to write file");
      fs::write(
        format!("{}/node-coordinates.txt", prefix),
        format!("{:?}", node_coordinates)
      ).expect("Unable to write file");
      
      let mut env_s = "TSP_INITIAL_COORDS='".to_string();
      for (_i, x, y) in node_coordinates.iter() {
        env_s += format!("{:.2},{:.2} ", x, y).as_str();
      }
      env_s += "'";

      fs::write(
        format!("{}/node-coordinates-env.txt", prefix),
        env_s
      ).expect("Unable to write file");
    }
    None => { }
  }
  
  return best_path;
}

