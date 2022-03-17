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

