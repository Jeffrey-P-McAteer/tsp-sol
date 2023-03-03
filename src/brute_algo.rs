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

use std::sync::{Mutex, RwLock, Arc};

use super::*;

use num::{Num, NumCast};

use permutohedron::LexicalPermutation;

pub fn solve(node_coordinates: &Vec<(usize, fp, fp)>, weights: &Vec<Vec<fp>>, save_run_prefix: Option<String>) -> Vec<usize> {
  /*let s1 = solve_mt(node_coordinates, weights, save_run_prefix.clone());
  let s2 = solve_st(node_coordinates, weights, save_run_prefix);
  
  // If s1 and s2 are different we have a huge problem
  assert_eq!(s1.len(), s2.len());
  for (a, b) in s1.iter().zip(s2.iter()) {
    assert_eq!(a, b);
  }

  return s2;
  */
  //solve_mt(node_coordinates, weights, save_run_prefix) // currently incorrect!
  solve_st(node_coordinates, weights, save_run_prefix)
}

pub fn solve_st(node_coordinates: &Vec<(usize, fp, fp)>, weights: &Vec<Vec<fp>>, save_run_prefix: Option<String>) -> Vec<usize> {
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

pub fn solve_mt(node_coordinates: &Vec<(usize, fp, fp)>, weights: &Vec<Vec<fp>>, save_run_prefix: Option<String>) -> Vec<usize> {
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

  //let mut best_path = Arc::new(Mutex::new( current_path.clone() ));
  //let mut best_path_dist = Arc::new(RwLock::new( compute_dist(weights, &current_path) ));

  let threads = num_cpus::get_physical();
  let num_permutations = factorial( current_path.len() );
  let permutations_per_t = num_permutations / threads;

  let mut thread_paths = vec![];
  // We move the permutation forward by permutations_per_t each step so threads do not need to re-do any work to get to their beginning
  for t in 0..threads {
    let begin_p = permutations_per_t * t;
    let end_p = permutations_per_t * (t+1);
    for _ in begin_p..end_p {
      if !current_path.next_permutation() {
        break;
      }
    }
    thread_paths.push( current_path.clone() );
  }

  let mut thread_best_paths = vec![];
  for t in 0..threads {
    thread_best_paths.push( current_path.clone() );
  }
  let thread_best_paths = Arc::new(Mutex::new(thread_best_paths));

  crossbeam::scope(|s| {
    let thread_paths = &thread_paths;
    for t in 0..threads {
      let thread_best_paths = thread_best_paths.clone(); // Each thread gets an atomic ref to the mutex
      s.spawn(move |_| {
        let begin_p = permutations_per_t * t;
        let end_p = permutations_per_t * (t+1);

        let mut current_path: Vec<usize> = (thread_paths[t]).clone();

        let mut best_path = current_path.clone();
        let mut best_path_dist = compute_dist(weights, &best_path);

        // // Increment permution until we hit this thread's chunk of work
        // for _ in 0..begin_p {
        //   if !current_path.next_permutation() {
        //     break;
        //   }
        // }

        let mut p = begin_p;
        
        loop {
          let this_dist = compute_dist(weights, &current_path);
          if this_dist < best_path_dist {
            best_path = current_path.clone();
            best_path_dist = this_dist;
          }
          
          if !current_path.next_permutation() || p > end_p {
            break;
          }

          p += 1;
        }

        // Finally get a lock & write our best path to the list
        loop {
          if let Ok(ref mut thread_best_paths) = thread_best_paths.try_lock() {
            thread_best_paths[t] = best_path;
            break;
          }
        }


      });
    }
  }).expect("Error joining crossbeam threads!");

  // Now we pick the best of each N threads best paths
  let thread_best_paths = thread_best_paths.lock().expect("Could not lock thread_best_paths");
  let mut best_path = current_path.clone();
  let mut best_dist = compute_dist(weights, &best_path);
  for t in 0..threads {
    let this_dist = compute_dist(weights, &thread_best_paths[t]);
    if this_dist < best_dist {
      best_path = thread_best_paths[t].clone();
      best_dist = this_dist;
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
  
  return best_path.to_vec();
}

/*pub fn factorial<N: Num + Ord + NumCast + Copy>(num: N) -> N{
    match num {
        n if n <= (0 as N) => (1 as N),
        n if n > (0 as N) => ((1 as N)..num+(1 as N)).product(),
    }
}*/

pub fn factorial(num: usize) -> usize {
    match num {
        n if n <= 0 => 1,
        n if n > 0 => (1..num+1).product(),
        _ => std::unimplemented!(),
    }
}

