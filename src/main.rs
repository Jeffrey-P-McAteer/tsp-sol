extern crate tsplib;
use tsplib::{EdgeWeightType, NodeCoord, Type};

extern crate image;
use image::{ImageBuffer, Rgb};

extern crate imageproc;
use imageproc::drawing::draw_line_segment_mut;

use std::fs::File;
use std::path::Path;
use std::io::{BufReader};
use std::env;
use std::io::prelude::*;
use std::f32;

fn usage() {
  println!(r#"Usage: ./tsp-sol path/to/berlin52.tsp
"#);
}

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() < 2 {
    usage();
    return;
  }
  
  let file_arg = args.get(1).unwrap();

  if ! Path::new(file_arg).exists() {
    println!("File does not exist: {}", file_arg);
    return;
  }

  let file = match File::open(file_arg) {
    Ok(f) => f,
    Err(e) => {
      println!("Cannot open {}: {}", file_arg, e);
      return;
    }
  };
  
  // Use tsp lib to parse file
  let instance = match tsplib::parse( BufReader::new(file) ) {
    Ok(i) => i,
    Err(e) => {
      println!("Error parsing tsplib file {}: {}", file_arg, e);
      return;
    }
  };
  
  let node_coordinates: Vec<(usize, f32, f32)> = match instance.node_coord {
    Some(node_c) => match node_c {
      NodeCoord::Two(vec_count_loc_loc) => vec_count_loc_loc,
      NodeCoord::Three(_we_dont_care) => {
        println!("3D TSP problems currently unsupported.");
        return;
      }
    },
    None => {
      println!("Err: no coordinates found in {}", file_arg);
      return;
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
  
  // If we have 3 or fewer points, we're done. min bound is O(1), good job folks.
  if weights.len() <= 3 {
    print!("Ideal trip order: ");
    for i in 0..weights.len() {
      print!("{}, ", i);
    }
    println!("");
    return;
  }
  
  // We begin with points 0, 1, and 2.
  // These will be overwritten in the largest-triangle-fining process
  let mut ordered_visits: Vec<usize> = vec![0, 1, 2]; // holds the path as a vector of indexes relating to the city number beginning at 0
  
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
    
    unordered_visits.remove(unordered_idx);
    ordered_visits.insert(ordered_idx, furthest_non_collected_point_i);
    
    center = compute_center(&ordered_visits, &node_coordinates);
  }
  
  { // Print solution
    println!("Solution distance: {}", compute_dist(&weights, &ordered_visits));
    print!("Solution order: ");
    for p in &ordered_visits {
      print!("{} ", *p);
    }
    println!("");
  }
  
  save_state_image("./out.png", &ordered_visits, &node_coordinates, &center);
  
}

fn compute_dist(weights: &Vec<Vec<f32>>, path: &Vec<usize>) -> f32 {
  let mut total: f32 = 0.0;
  for p_i in 0..path.len() {
    total += weights[p_i][(p_i+1) % path.len()]; // mod lets us wrap at end (p_i == len(), (p_i+1) % len == 0)
  }
  return total;
}

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

fn compute_furthest(path: &Vec<usize>, unordered: &Vec<usize>, weights: &Vec<Vec<f32>>, locations: &Vec<(usize, f32, f32)>, center: &(f32, f32))
  ->
  (usize /*point i*/, usize /*points idx in path*/, usize /*points idx in unordered*/)
{
  let mut furthest_i = 0;
  let mut unordered_idx = 0;
  let mut furthest_i_dist_from_center: f32 = 0.0;
  for i in path {
    if furthest_i == *i {
      furthest_i = (furthest_i+1) % locations.len();
    }
  }
  // now furthest_i is not in the path
  
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
  let furthest_i = furthest_i;
  let unordered_idx = unordered_idx;
  
  // Now determine shortest split & merge, set path_idx=
  let mut ideal_insert_dist_delta: f32 = f32::INFINITY;
  let mut path_idx = 0; // 0 indicates a split of the edge that runs between 0 -> 1
  
  for from_i in 0..path.len() {
    let to_i = (from_i+1) % path.len();
    let this_dist_delta: f32 = 
      (-weights[from_i][to_i]) +    // removed edge counts negative
      weights[from_i][furthest_i] + // add edge from -> new
      weights[furthest_i][to_i];    // add edge new -> end
    
    if this_dist_delta < ideal_insert_dist_delta {
      ideal_insert_dist_delta = this_dist_delta;
      path_idx = from_i;
    }
  }
  
  return (furthest_i, path_idx, unordered_idx);
}

fn save_state_image<I: Into<String>>(file_path: I, path: &Vec<usize>, locations: &Vec<(usize, f32, f32)>, center: &(f32, f32)) {
  let file_path = file_path.into();
  let (width, height) = (1200, 1200);
  let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height); // width, height
  
  let (smallest_x, largest_y, largest_x, smallest_y) = get_point_extents(locations);
  let x_range: f32 = largest_x - smallest_x;
  let y_range: f32 = largest_y - smallest_y;
  
  for loc in locations {
    let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.1, loc.2);
    
    // Set all location pixels to be red // r,g,b
    //image.get_pixel_mut(loc_x, loc_y).data = [255, 0, 0];
    circle_it(&mut image, loc_x, loc_y, [255, 0, 0]);
    
  }
  
  for pt_from in path {
    let pt_to = (pt_from+1) % locations.len();
    
    let from_loc = locations[*pt_from];
    let (from_loc_x,from_loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, from_loc.1, from_loc.2);
    
    let to_loc = locations[pt_to];
    let (pt_to_x,pt_to_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, to_loc.1, to_loc.2);
    
    draw_line_segment_mut(&mut image,
      (pt_to_x as f32,pt_to_y as f32), // start
      (from_loc_x as f32,from_loc_y as f32), // end
      Rgb([255, 255, 255])
    );
  }
  
  image.save(file_path).unwrap();
}

fn scale_xy(img_w: u32, img_h: u32, path_w: u32, path_h: u32, path_x_smallest: f32, path_y_smallest: f32, given_x: f32, given_y: f32) -> (u32, u32) {
  let mut img_x = (given_x - path_x_smallest) * ((img_w as f32 / path_w as f32) as f32);
  let mut img_y = (given_y - path_y_smallest) * ((img_h as f32 / path_h as f32) as f32);
  return (img_x as u32, img_y as u32);
}

// erm they're actually squares. Whoops.
fn circle_it(image: &mut ImageBuffer::<Rgb<u8>, Vec<u8>>, x: u32, y: u32, rgb: [u8; 3]) {
  let r = 10; // radius
  
  let x: i32 = x as i32; // UGH
  let y: i32 = y as i32;
  
  for x_off in (x-r)..(x+r) {
    for y_off in (y-r)..(y+r) {
      if x_off > 0 && x_off < image.width() as i32 && y_off > 0 && y_off < image.height() as i32 {
        image.get_pixel_mut(x_off as u32, y_off as u32).data = rgb;
      }
    }
  }
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

