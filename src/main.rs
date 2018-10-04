extern crate tsplib;
use tsplib::{EdgeWeightType, NodeCoord, Type};

extern crate image;
use image::{ImageBuffer, Rgb};

extern crate imageproc;
use imageproc::drawing::*;

extern crate rusttype;
use rusttype::{FontCollection, Scale};

extern crate rand;
use rand::prelude::*;

use std::fs::File;
use std::path::Path;
use std::io::{BufReader};
use std::env;
use std::io::prelude::*;
use std::f32;

mod brute_algo;
mod jeff_algo;

fn usage() {
  println!(r#"Usage: ./tsp-sol path/to/berlin52.tsp|delta
"#);
}

/// We will read in a problem & compute a weights matrix, the solver must return
/// a vector of the path to take from city index to index.
/// Solver function header:
///   fn solve(problem: &tsplib::Instance, weights: &Vec<Vec<f32>>) -> Vec<usize>

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() < 2 {
    usage();
    return;
  }
  
  let file_arg = args.get(1).unwrap();
  
  if file_arg == "delta" {
    delta(1000, 4, 8); // test the algorithm on a thousand generated cities, between 4-8 points each.
    return;
  }

  let (node_coordinates, weights) = match open_tsp_problem(file_arg.to_string()) {
    Some(stuff) => stuff,
    None => {
      return; // error message printed in open_tsp_problem
    }
  };
  
  let solution_p = jeff_algo::solve(&node_coordinates, &weights);
  println!("====== jeff_algo::solve ======");
  print_path_metadata(&solution_p, &weights);
}

fn delta(num_tests: usize, lower_city_size: usize, upper_city_size: usize) {
  let mut rng = thread_rng();
  for i in 0..num_tests {
    let city_size = rng.gen_range(lower_city_size, upper_city_size);
    println!("Delta testing {}/{}", i, num_tests);
    delta_test(city_size);
  }
}

fn delta_test(city_size: usize) {
  
}

fn print_path_metadata(path: &Vec<usize>, weights: &Vec<Vec<f32>>) {
  println!("Solution distance: {}", compute_dist(weights, path));
  print!("Solution order: ");
  for p in path {
    print!("{} ", *p);
  }
  println!("");
}

fn compute_dist(weights: &Vec<Vec<f32>>, path: &Vec<usize>) -> f32 {
  let mut total: f32 = 0.0;
  for p_i in 0..path.len() {
    let p  = path[p_i];
    let p2 = path[(p_i+1) % path.len()]; // mod lets us wrap at end (p_i == len(), (p_i+1) % len == 0)
    total += weights[p][p2];
  }
  return total;
}

fn open_tsp_problem(file_arg: String) -> Option<(Vec<(usize, f32, f32)>, Vec<Vec<f32>>)> {
  if ! Path::new(&file_arg).exists() {
    println!("File does not exist: {}", file_arg);
    return None;
  }

  let file = match File::open(file_arg.clone()) {
    Ok(f) => f,
    Err(e) => {
      println!("Cannot open {}: {}", file_arg, e);
      return None;
    }
  };
  
  // Use tsp lib to parse file
  let instance = match tsplib::parse( BufReader::new(file) ) {
    Ok(i) => i,
    Err(e) => {
      println!("Error parsing tsplib file {}: {}", file_arg, e);
      return None;
    }
  };
  
  let node_coordinates: Vec<(usize, f32, f32)> = match instance.node_coord {
    Some(node_c) => match node_c {
      NodeCoord::Two(vec_count_loc_loc) => vec_count_loc_loc,
      NodeCoord::Three(_we_dont_care) => {
        println!("3D TSP problems currently unsupported.");
        return None;
      }
    },
    None => {
      println!("Err: no coordinates found in {}", file_arg);
      return None;
    }
  };
  
  // Compute 2x matrix of edge weights (assumes 2d euclidian geometry)
  let mut weights: Vec<Vec<f32>> = Vec::with_capacity(node_coordinates.len());
  {
    for row_r in &node_coordinates {
      let mut row_weight_v: Vec<f32> = Vec::with_capacity(node_coordinates.len());
      for col_r in &node_coordinates {
        let weight: f32 = (
          (row_r.1 - col_r.1).powf(2.0) + // x1 + x2 squared
          (row_r.2 - col_r.2).powf(2.0)   // y1 + y2 squared
        ).sqrt();
        
        row_weight_v.push(weight);
      }
      weights.push(row_weight_v);
    }
  }
  
  println!("City has {} points", weights.len());
  // remember weights is 2d square matrix (could be triangle, meh.)
  
  return Some( (node_coordinates, weights) );
}

