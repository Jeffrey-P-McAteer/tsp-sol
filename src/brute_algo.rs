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
use std::sync::atomic::{AtomicBool, Ordering};

use std::collections::HashMap;

use super::*;

use num::{Num, NumCast};

use permutohedron::LexicalPermutation;

use once_cell::sync::Lazy;

use fasthash::*;
use fasthash::{FastHash, XXHasher};

use std::hash::{Hash, Hasher};
use std::io::{Read, Write};

type CityNum = usize;
type CityWeight = fp;
type CityXYCoord = fp;


// Maps a begin_permutation_num -> Vec<CityNum> coordinates.
// As long as the .len() of the shared Vec<CityNum>s is the same this will save
// a ton of work.
static PERMUTATIONS_CACHE: Lazy<Mutex<HashMap<CityNum, Vec<CityNum>, fasthash::RandomState<fasthash::xx::Hash64> >>> = Lazy::new(|| {
    let s = fasthash::RandomState::<fasthash::xx::Hash64>::new();
    Mutex::new( HashMap::with_hasher(s) )
});

pub fn solve(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<CityWeight>>, save_run_prefix: Option<String>) -> Vec<CityNum> {
  if let Some(cached_solution_vec) = get_cached_solution(node_coordinates) {
    return cached_solution_vec;
  }

  let best_path = if weights.len() < 7 {
    solve_st(node_coordinates, weights, 0, get_num_permutations(weights) ) // avoid thread overhead
  }
  else {
    solve_mt(node_coordinates, weights)
  };
  
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

  cache_solution(node_coordinates, &best_path);

  return best_path;
}

#[inline(always)]
fn cached_solution_key(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>) -> String {
  let mut s = XXHasher::default();
  for (node_num, node_x, node_y) in node_coordinates {
    let node_x: isize = ( node_x * 10000.0 ) as isize;
    let node_y: isize = ( node_y * 10000.0 ) as isize;

    node_num.hash(&mut s);
    node_x.hash(&mut s);
    node_y.hash(&mut s);
  }
  let hash_u64 = s.finish();

  format!("{:#08x}", hash_u64)
}

#[inline(always)]
fn cached_solution_file(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>) -> String {
  format!("target/_cached_solution_{}.txt", cached_solution_key(node_coordinates) )
}

fn get_cached_solution(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>) -> Option<Vec<CityNum>> {
  let cache_file = cached_solution_file(node_coordinates);
  let mut f = File::open(&cache_file).ok()?;
  let mut buffer = Vec::new();
  f.read_to_end(&mut buffer).ok()?;
  // buffer is full of bytes, decode to Vec<CityNum>
  let zero_vec = zerovec::ZeroVec::parse_byte_slice(&buffer).ok()?;
  let vec_u64: Vec<u64> = zero_vec.to_vec();

  // I hereby declare all u64 == usize for the machines running this code
  Some( unsafe { std::mem::transmute::<Vec<u64>,Vec<usize>>(vec_u64) } )
}

fn cache_solution(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, solution_best_path: &Vec<CityNum>) -> Option<()> {
  // I hereby declare all u64 == usize for the machines running this code
  let solution_best_path: &Vec<u64> = unsafe { std::mem::transmute::<&Vec<usize>, &Vec<u64>>(solution_best_path) };
  let zero_vec = zerovec::ZeroVec::from_slice_or_alloc(&solution_best_path);
  
  let zero_vec_bytes = zero_vec.into_bytes();
  let cache_file = cached_solution_file(node_coordinates);
  
  let mut file = std::fs::OpenOptions::new()
      .create(true) // To create a new file
      .write(true)
      // either use the ? operator or unwrap since it returns a Result
      .open(&cache_file).ok()?;

  file.write_all( &zero_vec_bytes.to_vec() ).ok()?;

  Some(())
}



// The mathematicians know num permutations == factorial of set, but I sure won't remember that.
#[inline(always)]
fn get_num_permutations<T>(current_path: &Vec<T>) -> CityNum {
  return factorial( current_path.len() );
}

#[inline(always)]
fn get_permutation_cache_key(begin_permutation_num: CityNum, num_weights: CityNum) -> CityNum {
  if num_weights > 99 { // because they would overlap
    panic!("Refusing to allow brute-force cache keys to be used with a graph 100+ cities large. Please change cache key algorithm to use larger graphs.");
  }
  return (begin_permutation_num * 100) + num_weights; // ensures overlapping begin_permutation_num across graph sizes do not collide.
}

pub fn solve_st(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<CityWeight>>, begin_permutation_num: CityNum, max_permutation_num: CityNum) -> Vec<CityNum> {
   let mut current_path = vec![];
   if let Some(cached_path) = PERMUTATIONS_CACHE.lock().unwrap().get( &get_permutation_cache_key(begin_permutation_num, weights.len()) ) {
      current_path = cached_path.clone();
   }
   else {
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

     for _ in 0..begin_permutation_num { // move UP to the first permutation idx
        current_path.next_permutation();
     }
   }
   // Now current_path is the correct permutation for this range
 
   let mut best_path = current_path.clone();
   let mut best_path_dist = compute_dist(weights, &best_path);
   
   let mut permutation_num = begin_permutation_num;
   loop {
     let this_dist = compute_dist(weights, &current_path);
     if this_dist < best_path_dist {
       best_path = current_path.clone();
       best_path_dist = this_dist;
     }
     permutation_num += 1;
     if permutation_num > max_permutation_num {
       break; // for multithreading purposes, this thread is done.
     }
     
     if !current_path.next_permutation() {
       break;
     }
 
   }
   
   return best_path;
}

pub fn solve_mt(node_coordinates: &Vec<(CityNum, CityXYCoord, CityXYCoord)>, weights: &Vec<Vec<CityWeight>>) -> Vec<CityNum> {
  let threads = num_cpus::get_physical();
  let num_permutations = get_num_permutations( weights );
  let permutations_per_t = num_permutations / threads;


  let permutations_remainder = num_permutations - (permutations_per_t * threads); // we just do this at the end of other threads work
  let a_thread_is_processing_remainder = Arc::new(AtomicBool::new(false));

  let mut thread_best_paths: Vec<Vec<CityNum>> = vec![];
  for t in 0..(threads + 1) {
    thread_best_paths.push(vec![]); // empty vec is sentinel value
  }

  let thread_best_paths = Arc::new(Mutex::new(thread_best_paths));

  // Populate cache for all begin_p values plus the threads * permutations_per_t beginning.
  let mut cache_current_path = vec![];
   for i in 0..weights.len() {
     cache_current_path.push(i);
   }
   // We, uh... need to go backwards until we hit the "first ordered permutation".
   // This was a bug in the original implementation which apparently just went from
   // random permutation index -> last sorted permutation.
   loop {
     if !cache_current_path.prev_permutation() {
       break;
     }
   }
  for t in 0..(threads+1) {
    let begin_p = permutations_per_t * t;
    let end_p = permutations_per_t * (t+1);
    let mut cache_ref = PERMUTATIONS_CACHE.lock().unwrap();
    if let Some(cached_path) = cache_ref.get(&get_permutation_cache_key(begin_p, weights.len())) {
      cache_current_path = cached_path.clone();
    }
    else {
      cache_ref.insert(get_permutation_cache_key(begin_p, weights.len()), cache_current_path.clone() );
      for _ in 0..permutations_per_t { // increase by permutations_per_t permutations & insert
          cache_current_path.next_permutation();
      }
      cache_ref.insert(get_permutation_cache_key(end_p, weights.len()), cache_current_path.clone() );
    }
  }


  crossbeam::scope(|s| {
    for t in 0..threads {
      let thread_best_paths = thread_best_paths.clone(); // Each thread gets an atomic ref to the mutex
      let a_thread_is_processing_remainder = a_thread_is_processing_remainder.clone(); // allow move of AtomicBool through our Arc
      s.spawn(move |_| {
        let begin_p = permutations_per_t * t;
        let end_p = permutations_per_t * (t+1);

        let best_t_path = solve_st(node_coordinates, weights, begin_p, end_p );
        // Finally get a lock & write our best path to the list
        loop {
          if let Ok(ref mut thread_best_paths) = thread_best_paths.try_lock() {
            thread_best_paths[t] = best_t_path;
            break;
          }
        }

        // Has someone already begun processing the remainder job?
        if !a_thread_is_processing_remainder.load(Ordering::Relaxed) {
          a_thread_is_processing_remainder.store(true, Ordering::Relaxed);
          let best_t_path = solve_st(node_coordinates, weights, permutations_per_t * threads, num_permutations);
          // Same deal, get lock & write to final index.
          loop {
            if let Ok(ref mut thread_best_paths) = thread_best_paths.try_lock() {
              thread_best_paths[threads] = best_t_path;
              break;
            }
          }
        }

      });
    }
  }).expect("Error joining crossbeam threads!");

  // Now we pick the best of each N threads best paths
  let thread_best_paths = thread_best_paths.lock().expect("Could not lock thread_best_paths");
  let mut best_path = thread_best_paths[0].clone();
  let mut best_dist = compute_dist(weights, &best_path);
  for t in 0..(threads+1) {
    let this_dist = compute_dist(weights, &thread_best_paths[t]);
    if this_dist < best_dist {
      best_path = thread_best_paths[t].clone();
      best_dist = this_dist;
    }
  }
  
  return best_path.to_vec();
}

/*pub fn factorial<N: Num + Ord + NumCast + Copy>(num: N) -> N{
    match num {
        n if n <= (0 as N) => (1 as N),
        n if n > (0 as N) => ((1 as N)..num+(1 as N)).product(),
    }
}*/

#[inline(always)]
pub fn factorial(num: CityNum) -> CityNum {
    match num {
        n if n <= 0 => 1,
        n if n > 0 => (1..num+1).product(),
        _ => std::unimplemented!(),
    }
}

