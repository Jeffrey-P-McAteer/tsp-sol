/**
 *  tsp-sol - an experimental environment for traveling salesman solution analysis
 *  Copyright (C) 2019  Jeffrey McAteer <jeffrey.p.mcateer@gmail.com>
 *  
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
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

extern crate permutohedron;

use std::fs;
use std::fs::{File,create_dir};
use std::path::Path;
use std::io::{BufReader};
use std::env;
use std::io::prelude::*;
use std::f32;

mod brute_algo;
mod jeff_algo;

fn usage() {
  println!(r#"Usage: ./tsp-sol path/to/berlin52.tsp|delta|selective

Passing a single file (tsp/berlin52.tsp) will run JeffAlgo on it and pring the size and solution path.

delta will cause 1000 runs using both JeffAlgo and BruteAlgo, incorrect JeffAlgo runs will be dumped to the ./views directory.

selective loops throuh increasingly large cities and exits when JeffAlgo does not match BruteAlgo.

"#);
}

/// We will read in a problem & compute a weights matrix, the solver must return
/// a vector of the path to take from city index to index.
/// Solver function header:
///   pub fn solve(node_coordinates: Vec<(usize, f32, f32)>, weights: &Vec<Vec<f32>>) -> Vec<usize>

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() < 2 {
    usage();
    return;
  }
  
  let file_arg = args.get(1).unwrap();
  
  if file_arg == "delta" {
    let num = 1000;
    let num_failed = delta(num, 4, 8); // test the algorithm on a thousand generated cities, between 4-8 points each.
    println!("Failed {} out of {}", num_failed, num);
    return;
  }
  
  if file_arg == "selective" {
    selective(); // generate increasing city size until failure (jeff() != brute()), then go back and map a large range of points
    return;
  }

  let (node_coordinates, weights) = match open_tsp_problem(file_arg.to_string()) {
    Some(stuff) => stuff,
    None => {
      return; // error message printed in open_tsp_problem
    }
  };
  
  let solution_p = jeff_algo::solve(&node_coordinates, &weights, None);
  println!("====== jeff_algo::solve ======");
  print_path_metadata(&solution_p, &weights);
  
  //let solution_p = brute_algo::solve(&node_coordinates, &weights, None);
  //println!("====== brute_algo::solve ======");
  //print_path_metadata(&solution_p, &weights);
}

fn delta(num_tests: usize, lower_city_size: usize, upper_city_size: usize) -> usize {
  let mut rng = thread_rng();
  let mut total_failed: usize = 0;
  for i in 0..num_tests {
    let city_size = rng.gen_range(lower_city_size, upper_city_size);
    println!("Delta testing {}/{}", i, num_tests);
    if ! delta_test(city_size) {
      total_failed += 1;
    }
  }
  return total_failed;
}

fn delta_test(city_size: usize) -> bool {
  let (node_coordinates, weights) = gen_tsp_problem(city_size, 0.0, 10.0, 0.0, 10.0);
  
  let jeff_sol = jeff_algo::solve(&node_coordinates, &weights, None);
  let brute_sol = brute_algo::solve(&node_coordinates, &weights, None);
  
  let jeff_sol_len = compute_dist(&weights, &jeff_sol);
  let brute_sol_len = compute_dist(&weights, &brute_sol);
  
  let distance_diff = jeff_sol_len - brute_sol_len;
  
  if distance_diff.abs() > 0.01 { // account for floating point errors
    // re-do test, saving results
    let r_test_num: usize = rand::thread_rng().gen_range(0, 10000000);
    
    let prefix_dir = format!("./views/{:02}-{}/", weights.len(), r_test_num);
    jeff_algo::solve(&node_coordinates, &weights, Some(prefix_dir.clone()));
    brute_algo::solve(&node_coordinates, &weights, Some(prefix_dir.clone()));
    return false;
  }
  return true;
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

fn gen_tsp_problem(num_points: usize, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> (Vec<(usize, f32, f32)>, Vec<Vec<f32>>) {
  let mut rng = rand::thread_rng();
  let mut node_coordinates: Vec<(usize, f32, f32)> = vec![];
  
  for i in 0..num_points {
    node_coordinates.push(
      (i, rng.gen_range(min_x, max_x), rng.gen_range(min_y, max_y))
    );
  }
  
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
  
  return (node_coordinates, weights);
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
  let weights = compute_weight_coords(&node_coordinates);
  
  println!("City has {} points", weights.len());
  // remember weights is 2d square matrix (could be triangle, meh.)
  
  return Some( (node_coordinates, weights) );
}

// Meh used in imagery

fn compute_center(path: &Vec<usize>, locations: &Vec<(usize, f32, f32)>) -> (f32, f32) {
  let mut x_tot: f32 = 0.0;
  let mut y_tot: f32 = 0.0;
  
  for p in path {
    x_tot += locations[*p].1;
    y_tot += locations[*p].2;
  }
  
  x_tot /= path.len() as f32;
  y_tot /= path.len() as f32;
  return (x_tot, y_tot);
}

// Shared imagery functions

fn save_state_image<I: Into<String>>(file_path: I, path: &Vec<usize>, locations: &Vec<(usize, f32, f32)>, center: &(f32, f32)) {
  let file_path = file_path.into();
  let (width, height) = (600, 600);
  let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width + 5, height + 5); // width, height
  
  let (smallest_x, largest_y, largest_x, smallest_y) = get_point_extents(locations);
  let x_range: f32 = largest_x - smallest_x;
  let y_range: f32 = largest_y - smallest_y;
  
  for i in 0..locations.len() {
    let loc = locations[i];
    let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.1, loc.2);
    
    // Set all location pixels to be red // r,g,b
    //image.get_pixel_mut(loc_x, loc_y).data = [255, 0, 0];
    //circle_it(&mut image, loc_x, loc_y, [255, 0, 0]);
    draw_hollow_circle_mut(&mut image, (loc_x as i32, loc_y as i32), 10 /*radius*/, Rgb([255, 0, 0]));
    
    // Also draw an index number
    let font = Vec::from( include_bytes!("/usr/share/fonts/noto/NotoSans-Bold.ttf") as &[u8] );
    let font = FontCollection::from_bytes(font).unwrap().into_font().unwrap();
    
    let font_height = 14.0;
    let font_scale = Scale { x: font_height, y: font_height };
    draw_text_mut(&mut image, Rgb([200, 200, 255]), loc_x as u32, loc_y as u32, font_scale, &font, format!("{}", i).as_str());
  }
  
  for i in 0..path.len() {
    let pt_from = path[i];
    let pt_to =   path[(i+1) % path.len()];
    //println!("pt_from = {}, pt_to = {}", pt_from, pt_to);
    
    let from_loc = locations[pt_from];
    let (from_loc_x,from_loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, from_loc.1, from_loc.2);
    
    let to_loc = locations[pt_to];
    let (pt_to_x,pt_to_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, to_loc.1, to_loc.2);
    //println!("Going from {} to {}", pt_from, pt_to);
    
    draw_line_segment_mut(&mut image,
      (pt_to_x as f32,pt_to_y as f32), // start
      (from_loc_x as f32,from_loc_y as f32), // end
      Rgb([200, 200, 200])
    );
  }
  
  // center is green cross
  let (center_img_x, center_img_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, center.0, center.1);
  draw_cross_mut(&mut image, Rgb([0, 255, 0]), center_img_x as i32, center_img_y as i32);
  
  image.save(file_path).unwrap();
}

fn scale_xy(img_w: u32, img_h: u32, path_w: u32, path_h: u32, path_x_smallest: f32, path_y_smallest: f32, given_x: f32, given_y: f32) -> (u32, u32) {
  let mut img_x = (given_x - path_x_smallest) * ((img_w as f32 / path_w as f32) as f32);
  let mut img_y = (given_y - path_y_smallest) * ((img_h as f32 / path_h as f32) as f32);
  if img_x < 5.0 {
    img_x = 5.0;
  }
  if img_x > (img_w-5) as f32 {
    img_x = (img_w-5) as f32;
  }
  if img_y < 5.0 {
    img_y = 5.0;
  }
  if img_y > (img_h-5) as f32 {
    img_y = (img_h-5) as f32;
  }
  return (img_x as u32, img_y as u32);
}

// returns smallestX, largestY, largestX, smallestY
fn get_point_extents(locations: &Vec<(usize, f32, f32)>) -> (f32, f32, f32, f32) {
  let mut smallest_x = f32::INFINITY;
  let mut largest_y = f32::NEG_INFINITY;
  let mut largest_x = f32::NEG_INFINITY;
  let mut smallest_y = f32::INFINITY;
  for loc in locations {
    let x = loc.1;
    let y = loc.2;
    if x < smallest_x {
      smallest_x = x;
    }
    if x > largest_x {
      largest_x = x;
    }
    if y < smallest_y {
      smallest_y = y;
    }
    if y > largest_y {
      largest_y = y;
    }
  }
  return (smallest_x, largest_y, largest_x, smallest_y);
}

fn compute_weight_coords(node_coordinates: &Vec<(usize, f32, f32)>) -> Vec<Vec<f32>> {
  // Compute 2x matrix of edge weights (assumes 2d euclidian geometry)
  let mut weights: Vec<Vec<f32>> = Vec::with_capacity(node_coordinates.len());
  {
    for row_r in node_coordinates {
      let mut row_weight_v: Vec<f32> = Vec::with_capacity(node_coordinates.len());
      for col_r in node_coordinates {
        let weight: f32 = (
          (row_r.1 - col_r.1).powf(2.0) + // x1 + x2 squared
          (row_r.2 - col_r.2).powf(2.0)   // y1 + y2 squared
        ).sqrt();
        
        row_weight_v.push(weight);
      }
      weights.push(row_weight_v);
    }
  }
  return weights;
}

fn selective() {
  println!("Performing selective failure...");
  // Bounding box for all points
  let x_min_bound: f32 = 0.0;
  let x_max_bound: f32 = 15.0;
  let y_min_bound: f32 = 0.0;
  let y_max_bound: f32 = 15.0;
  
  let bound_granularity = 0.25; // step size with which to make grid points after failure
  
  let x_min: f32 = 5.0;
  let x_max: f32 = 10.0;
  let y_min: f32 = 5.0;
  let y_max: f32 = 10.0;
  
  let mut rng = rand::thread_rng();
  let mut node_coordinates: Vec<(usize, f32, f32)> = vec![];
  
  // Just add 3 to begin with
  for i in 0..3 {
    let new_r_city = (
      i,
      rng.gen_range(x_min, x_max),
      rng.gen_range(y_min, y_max),
    );
    node_coordinates.push(new_r_city);
  }
  
  // If we hit 11 cities without a failure we'll recurse and start from 3 again.
  for city_num in 3..9 {
    let new_r_city = (
      city_num,
      rng.gen_range(x_min, x_max),
      rng.gen_range(y_min, y_max),
    );
    node_coordinates.push(new_r_city); // we can pop() if we fail
    
    let city_weights = compute_weight_coords(&node_coordinates);
    
    let jeff_sol = jeff_algo::solve(&node_coordinates, &city_weights, None);
    let brute_sol = brute_algo::solve(&node_coordinates, &city_weights, None);
    
    let jeff_sol_len = compute_dist(&city_weights, &jeff_sol);
    let brute_sol_len = compute_dist(&city_weights, &brute_sol);
    let distance_diff = jeff_sol_len - brute_sol_len;
    
    if distance_diff.abs() > 0.01 { // account for floating point errors
      println!("We have broken jeff_algo at {} points!", city_num+1);
      // we have added a city which breaks things!
      node_coordinates.pop();
      let city_weights = compute_weight_coords(&node_coordinates);
      
      // Now we have a city right before our failure.
      
      // Save the correct solution
      brute_algo::solve(&node_coordinates, &city_weights, Some("./views/selective/".to_string()));
      jeff_algo::solve(&node_coordinates, &city_weights, Some("./views/selective/".to_string()));
      
      // compute a 2d matrix of points and plot blue if they result in correct, red if they do not.
      perform_matrix_image_gen("./views/selective-map.png", node_coordinates, city_weights, );
      
      
      return;
    }
  }
  
  println!("Failed to break after 10, resetting...");
  selective();
  
}

fn perform_matrix_image_gen<S: Into<String>>(img_path: S, node_coordinates: Vec<(usize, f32, f32)>, city_weights: Vec<Vec<f32>>) {
  
  
  
}


