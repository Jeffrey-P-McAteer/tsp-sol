
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

/**
 *  tsp-sol - an experimental environment for traveling salesman solution analysis
 *  Copyright (C) 2023  Jeffrey McAteer <jeffrey@jmcateer.com>
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

use tsplib::{NodeCoord};

use image::{RgbImage, Rgb, GenericImage};

use imageproc::drawing::*;

use rusttype::{Font, Scale};

use rand::prelude::*;

use permutohedron;

use threadpool::ThreadPool;
use num_cpus;

use once_cell::sync::Lazy;

use wgpu::util::DeviceExt;

use std::fs;
use std::fs::{File,create_dir};
use std::path::Path;
use std::io::{BufReader,Write};
use std::sync::{Mutex};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::env;
//use std::io::prelude::*;
use std::f32;
use std::f64;

mod brute_algo;
mod jeff_algo;

mod parabolics;

#[allow(non_camel_case_types)]
//pub type fp = f64;
pub type fp = f32;


pub type CityNum = usize;
pub type CityWeight = fp;
pub type CityXYCoord = fp;

// fp numbers within this distance are considered equal
#[allow(non_upper_case_globals)]
const fp_epsilon: fp = 0.0001;

#[allow(non_upper_case_globals)]
pub const x_min_bound: fp = 0.0;
#[allow(non_upper_case_globals)]
pub const x_max_bound: fp = 15.0;
#[allow(non_upper_case_globals)]
pub const y_min_bound: fp = 0.0;
#[allow(non_upper_case_globals)]
pub const y_max_bound: fp = 15.0;

#[allow(non_upper_case_globals)]
pub const x_min: fp = 3.0;
#[allow(non_upper_case_globals)]
pub const x_max: fp = 12.0;
#[allow(non_upper_case_globals)]
pub const y_min: fp = 3.0;
#[allow(non_upper_case_globals)]
pub const y_max: fp = 12.0;


pub const HTML_BEGIN: &'static str = r#"
<!DOCTYPE html>
  <head>
    <style>
      div > * {
        display: none;
        width: 450px;
        background-color: Canvas ;
        position: relative;
        top:-120px;
        left:10px;
      }
      div:hover > * {
        display: block;
        pointer-events:none;
      }
      html, body {
        background: #c0c0c0;
      }
    </style>
    <script>
      /* test w/ draw_path( document.querySelectorAll('div')[5] ) */
      function draw_path(clicked_elm) {
        console.log('draw_path', clicked_elm);
        /*event = event || window.event;
        var clicked_elm = event.currentTarget || event.target;*/
        var path_coords_s = clicked_elm.getAttribute("c").split(" ");
        console.log(path_coords_s);
        var path_coords = [];
        for (var i=0; i<path_coords_s.length; i+= 1) {
          try {
            var coords_s = path_coords_s[i].split(",");
            if (coords_s.length < 2) {
              continue;
            }
            path_coords.push(
              [ parseFloat(coords_s[0]), parseFloat(coords_s[1]) ]
            );
          }
          catch (e) {
            console.log(e);
          }
        }
        console.log(path_coords);
        var canvas_elm = document.getElementById("overlay-canvas");
        var ctx = canvas_elm.getContext("2d");
        ctx.clearRect(0, 0, canvas_elm.width, canvas_elm.height);

        try {
          var initial_coords_s = document.getElementById("initial-sol").getAttribute("c").split(" ");
          var initial_coords = [];
          for (var i=0; i<initial_coords_s.length; i+= 1) {
            try {
              var coords_s = initial_coords_s[i].split(",");
              if (coords_s.length < 2) {
                continue;
              }
              initial_coords.push(
                [ parseFloat(coords_s[0]), parseFloat(coords_s[1]) ]
              );
            }
            catch (e) {
              console.log(e);
            }
          }
          ctx.lineWidth = 1;
          ctx.strokeStyle = 'white';
          ctx.setLineDash([]);
          var last_coords = initial_coords[initial_coords.length-1];
          for (var i=0; i<initial_coords.length; i+= 1) {
            var dis_coords = initial_coords[i];
            ctx.beginPath();
            ctx.moveTo(last_coords[0], last_coords[1]);
            ctx.lineTo(dis_coords[0], dis_coords[1]);
            ctx.stroke();
            last_coords = dis_coords;
          }
        }
        catch (e) {
          console.log(e);
        }

        ctx.lineWidth = 2;
        ctx.strokeStyle = 'black';
        ctx.setLineDash([10,10]);
        var last_coords = path_coords[path_coords.length-1];
        for (var i=0; i<path_coords.length; i+= 1) {
          var dis_coords = path_coords[i];
          ctx.beginPath();
          ctx.moveTo(last_coords[0], last_coords[1]);
          ctx.lineTo(dis_coords[0], dis_coords[1]);
          ctx.stroke();
          last_coords = dis_coords;
        }
      }
    </script>
  </head>
  <body>
"#;
pub const HTML_END: &'static str = r#"
  <canvas id="overlay-canvas" width="1450px" height="1450px" style="position:absolute;top:0;left:0;pointer-events:none;"/>
</body>
"#;
pub const HTML_POINT_SCALE: fp = 100.0 as fp;

fn usage() {
  println!(r#"Usage: ./tsp-sol path/to/berlin52.tsp|delta|selective|spray

Passing a single file (tsp/berlin52.tsp) will run JeffAlgo on it and print the size and solution path.

delta will cause 1000 runs using both JeffAlgo and BruteAlgo, incorrect JeffAlgo runs will be dumped to the ./views directory.

selective loops throuh increasingly large cities and exits when JeffAlgo does not match BruteAlgo.

spray requires 2 numbers after it, the N-1 size of the city and a resolution to spray over. `spray` also takes
      the initial N-1 city coordinates from the environment variable TSP_INITIAL_COORDS. An example
      is TSP_INITIAL_COORDS='2.5,8.5 7.5,8.5 12.5,8.5 7.5,9.0' ./tsp-sol spray 4 0.2.
      This creates a city with the above coordinates, then for every point 0.2 units apart in
      the bounding box it attempts to insert it using both JeffAlgo and BruteAlgo. If the tours
      generated by JeffAlgo and BruteAlgo match a green pixel will be plotted on ./views/spray.png,
      if they do not match a red pixel is plotted. This may be used to graphically show where
      JeffAlgo fails to uphold the hamiltonian cycle invariant from city size N to N+1.

pattern-scan N granularity
  pattern-scan needs the number of cities to consider and the granularity of the grid of
  N+1 points to lay on top; produces a graph showing all identical tours in the same color.

multi-pattern-scan N pattern-granularity num-steps-to-interop
  multi-pattern-scan needs the number of cities to consider, the granularity of the grid of
  N+1 points to lay on top, and a step number.
  The step number is how many steps we use to move a city from its arrangement
  defined by TSP_INITIAL_COORDS to TSP_ENDING_COORDS, for example a value of 3
  would produce 1 pattern-scan result considering TSP_INITIAL_COORDS,
  1 pattern-scan result for a city w/ points at the midpoint of each of the 2 arrangements,
  and finally 1 pattern-scan result considering TSP_ENDING_COORDS.

spray-pattern-search N pattern-granularity max-sprays-to-perform
  spray-pattern-search needs the number of cities to consider, the granularity of the grid of
  N+1 points to lay on top, and the number of sprays to perform.
  For each spray, generates a random N-sized city and performs pattern-scan of the given pattern-granularity,
  recording additional details along the way.

  The goal of this operation is to test conjectures about relationships between edge weights and
  optimal tour patterns, which get written in spray_pattern_search().



"#);
}

/// We will read in a problem & compute a weights matrix, the solver must return
/// a vector of the path to take from city index to index.
/// Solver function header:
///   pub fn solve(node_coordinates: Vec<(usize, fp, fp)>, weights: &Vec<Vec<fp>>) -> Vec<usize>

fn main() {
  let begin_time = std::time::Instant::now();
  timed_main();
  let exec_duration = begin_time.elapsed();
  println!("=== Elapsed time: {:?} ===", exec_duration);

  // Flush any brute algo cache we may have
  if brute_algo::use_brute_cache_env_val() {
    if let Err(e) = brute_algo::PICKLE_DB.get_mut().dump() {
      eprintln!("Error saving brute cache: {:?}", e);
    }
    if let Err(e) = brute_algo::MULTI_PICKLE_DB.get_mut().dump() {
      eprintln!("Error saving multi brute cache: {:?}", e);
    }
  }

}

fn timed_main() {
  let args: Vec<_> = env::args().collect();
  if args.len() < 2 {
    usage();
    return;
  }

  if cfg!(windows) {
    attempt_to_raise_priority();
  }

  let thread_pool = ThreadPool::new( num_cpus::get_physical() );
  println!("Brute force algo thread pool size: {}", thread_pool.max_count());

  // Initialize GPU

  // Grab largest device, report sizes, and pass this to downstream funcs which may either use the
  // thread_pool OR the emu device object to schedule work across
  let mut gpu_adapter = get_best_gpu();
  if let Some(ref adapter) = gpu_adapter {
    println!("GPU device = {:?}", adapter.get_info());
  }
  else {
    println!("NO GPU");
  }

  let file_arg = args.get(1).unwrap();

  let mut use_jalgo = true;
  let mut use_brute = false;
  let mut write_solution_out_to_views = false;
  for arg in &args {
    if arg == "jalgo" {
      println!("Enabling jalgo...");
      use_jalgo = true;
    }
    if arg == "brute" {
      println!("Enabling brute...");
      use_brute = true;
    }
    if arg == "no-jalgo" {
      println!("Disabling jalgo...");
      use_jalgo = false;
    }
    if arg == "no-brute" {
      println!("Brute jalgo...");
      use_brute = false;
    }
    if arg == "view" {
      write_solution_out_to_views = true;
    }
    if arg == "no-view" {
      write_solution_out_to_views = false;
    }
  }

  if file_arg == "pattern-scan" {
    // Given a city of points, add one more in a grid and
    // store a color based on the optimal point arrangement.
    // Sections w/ the same ideal solution path will be grouped
    // as the same color.
    pattern_scan(
      args.get(2).unwrap_or(&"5".to_string()).parse().unwrap(), // given number OR 5
      args.get(3).unwrap_or(&"0.25".to_string()).parse().unwrap(), // given number OR 0.25
      "views/pattern-scan.png",
      &thread_pool, &mut gpu_adapter
    );
    return;
  }

  if file_arg == "multi-pattern-scan" {
    // Same as pattern-scan, but take in 2 cities and perform
    // pattern_scan in steps
    multi_pattern_scan(
      args.get(2).unwrap_or(&"5".to_string()).parse().unwrap(), // given number OR 5 - number of cities
      args.get(3).unwrap_or(&"0.25".to_string()).parse().unwrap(), // given number OR 0.25 - resolution to generate a SINGLE multi pattern at
      args.get(4).unwrap_or(&"10".to_string()).parse().unwrap(), // number of steps to put between 2 cities, aka total number of pattern_scans to run.
      &thread_pool, &mut gpu_adapter
    );
    return;
  }

  if file_arg == "spray-pattern-search" {
    spray_pattern_search(
      args.get(2).unwrap_or(&"5".to_string()).parse().unwrap(), // given number OR 5 - number of cities
      args.get(3).unwrap_or(&"0.25".to_string()).parse().unwrap(), // given number OR 0.25 - resolution to generate a SINGLE multi pattern at
      args.get(4).unwrap_or(&"100".to_string()).parse().unwrap(), // number of sprays to perform
      &thread_pool, &mut gpu_adapter
    );
    return;
  }

  if file_arg == "delta" {
    let num = 1000;
    let num_failed = delta(num, 4, 8, &thread_pool); // test the algorithm on a thousand generated cities, between 4-8 points each.
    println!("Failed {} out of {}", num_failed, num);
    return;
  }

  if file_arg == "selective" {
    // generate increasing city size until failure (jeff() != brute()), then go back and map a large range of points
    let max_cities_to_test: usize = args.get(2).unwrap_or(&"11".to_string()).parse().unwrap();  // arg after "selective" OR 11
    let num_to_test_before: usize = args.get(3).unwrap_or(&"2".to_string()).parse().unwrap();  // arg after "max_cities_to_test" OR 1
    let mut min_cities_to_ignore = max_cities_to_test - num_to_test_before;
    if min_cities_to_ignore >= max_cities_to_test {
      println!("Invalid num_to_test_before passed ({}), resetting min_cities_to_ignore from {} to {}", num_to_test_before, min_cities_to_ignore, max_cities_to_test - 1);
      min_cities_to_ignore = max_cities_to_test - 1;
    }
    selective(
      min_cities_to_ignore,
      max_cities_to_test,
      &thread_pool, &mut gpu_adapter
    );
    return;
  }

  if file_arg == "spray" {
    // Generate random N points then add a grid of points and track where insertion
    // results in a non-optimal path.
    spray(
      args.get(2).unwrap_or(&"5".to_string()).parse().unwrap(), // given number OR 5
      args.get(3).unwrap_or(&"0.25".to_string()).parse().unwrap(), // given number OR 0.25
      &thread_pool, &mut gpu_adapter
    );
    return;
  }

  let (node_coordinates, weights) = match open_tsp_problem(file_arg.to_string()) {
    Some(stuff) => stuff,
    None => {
      return; // error message printed in open_tsp_problem
    }
  };

  // First dump the environment variable we'd need to set to scan this city,
  // useful for going from .tsp file -> research
  let mut env_s = "TSP_INITIAL_COORDS='".to_string();
  for (_i, x, y) in node_coordinates.iter() {
    env_s += format!("{:.2},{:.2} ", x, y).as_str();
  }
  env_s += "'";
  println!("{}", env_s);

  if use_jalgo {
    let solution_p = if write_solution_out_to_views {
      jeff_algo::solve(&node_coordinates, &weights, Some( "./views/tsp_problem".to_string() ))
    }
    else {
      jeff_algo::solve(&node_coordinates, &weights, None)
    };
    println!("====== jeff_algo::solve ======");
    print_path_metadata(&solution_p, &weights);
  }

  if use_brute {
    let solution_p = if write_solution_out_to_views {
      let all_solutions = brute_algo::solve_all(&node_coordinates, &weights, Some( "./views/tsp_problem".to_string() ), &thread_pool);
      all_solutions[0].clone()
    }
    else {
      brute_algo::solve(&node_coordinates, &weights, None, &thread_pool)
    };
    println!("====== brute_algo::solve ======");
    print_path_metadata(&solution_p, &weights);
  }

}

fn get_best_gpu() -> Option<wgpu::Adapter> {
  let preferred_device_name = std::env::var("PREF_GPU");

  // Check for NO gpu set; mostly for debugging
  if let Ok(ref preferred_device_name) = preferred_device_name {
    if preferred_device_name.contains("NONE") || preferred_device_name.contains("None") || preferred_device_name.contains("none") {
      return None;
    }
  }

  let mut print_adapter_infos = false;
  if let Ok(ref preferred_device_name) = preferred_device_name {
    if preferred_device_name.contains("dump") || preferred_device_name.contains("print") || preferred_device_name.contains("list") {
      print_adapter_infos = true;
    }
  }
  let print_adapter_infos = print_adapter_infos;

  let mut backend = wgpu::Backends::VULKAN;
  if let Ok(backend_name) = std::env::var("PREF_BACKEND") {
    if backend_name.contains("vulkan") {
      backend = wgpu::Backends::VULKAN;
    }
    else if backend_name.contains("gl") {
      backend = wgpu::Backends::GL;
    }
    else if backend_name.contains("all") {
      backend = wgpu::Backends::all();
    }
  }

  let mut i_flags = wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION;
  if let Ok(inst_flags) = std::env::var("INST_FLAGS") {
    if inst_flags.contains("empty") || inst_flags.contains("none") {
      i_flags = wgpu::InstanceFlags::empty();
    }
  }

  let adapters = wgpu::Instance::new(wgpu::InstanceDescriptor {
    // backends: wgpu::Backends::all(),
    backends: backend,

    // flags: wgpu::InstanceFlags::empty(),
    flags: i_flags,

    dx12_shader_compiler: wgpu::Dx12Compiler::Fxc, // windorks only concern

    gles_minor_version: wgpu::Gles3MinorVersion::Automatic,

  });

  if print_adapter_infos {
    println!("============ GPUs Detected ==================");
    for adapter in adapters.enumerate_adapters(wgpu::Backends::VULKAN) {
      let info = adapter.get_info();
      println!("Adapter info = {:?}", info);
    }
    println!("============ GPUs Detected ==================");
  }

  for adapter in adapters.enumerate_adapters(wgpu::Backends::VULKAN) {
    let info = adapter.get_info();
    if let Ok(ref preferred_device_name) = preferred_device_name {
      if info.name.contains(preferred_device_name) {
        return Some(adapter);
      }
    }
    else {
      if info.device_type == wgpu::DeviceType::DiscreteGpu {
        return Some(adapter);
      }
    }
  }

  // We didn't get our preferred, return the first DiscreteGpu
  for adapter in adapters.enumerate_adapters(wgpu::Backends::VULKAN) {
    let info = adapter.get_info();
    if info.device_type == wgpu::DeviceType::DiscreteGpu {
      return Some(adapter);
    }
  }

  // Just grab the first one
  for adapter in adapters.enumerate_adapters(wgpu::Backends::VULKAN) {
    return Some(adapter);
  }
  // If no devices,
  return None;
}

fn attempt_to_raise_priority() {
  use std::process;
  use std::process::{Command, Stdio};
  let our_pid = process::id();
  let psutil_script = if cfg!(windows) {
    //format!("import psutil ; pid={our_pid} ; p=psutil.Process(pid) ; p.cpu_affinity([0]) ; p.nice(psutil.HIGH_PRIORITY_CLASS)", our_pid=our_pid)
    //format!("import psutil ; pid={our_pid} ; p=psutil.Process(pid) ; p.nice(psutil.HIGH_PRIORITY_CLASS)", our_pid=our_pid)
    format!("import psutil ; pid={our_pid} ; p=psutil.Process(pid) ; p.nice(psutil.HIGH_PRIORITY_CLASS) ; p.cpu_affinity([x for x in range(0, psutil.cpu_count(logical=True)) if x % int(psutil.cpu_count(logical=True) / psutil.cpu_count(logical=False)) == 0 ])", our_pid=our_pid)
  }
  else {
    //format!("import psutil ; pid={our_pid} ; p=psutil.Process(pid) ; p.cpu_affinity([0]) ; p.nice(-5)", our_pid=our_pid)
    format!("import psutil ; pid={our_pid} ; p=psutil.Process(pid) ;  p.nice(-5)", our_pid=our_pid)
  };
  let res = Command::new("python").args(&[
    "-c", &psutil_script
  ])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .stdin(Stdio::null())
    .spawn();
}

fn delta(num_tests: usize, lower_city_size: usize, upper_city_size: usize, thread_pool: &ThreadPool) -> usize {
  let mut rng = thread_rng();
  let mut total_failed: usize = 0;
  for i in 0..num_tests {
    let city_size = rng.gen_range(lower_city_size, upper_city_size);
    println!("Delta testing {}/{}", i, num_tests);
    if ! delta_test(city_size, thread_pool) {
      total_failed += 1;
    }
  }
  return total_failed;
}

fn delta_test(city_size: usize, thread_pool: &ThreadPool) -> bool {
  let (node_coordinates, weights) = gen_tsp_problem(city_size, 0.0, 10.0, 0.0, 10.0);

  let jeff_sol = jeff_algo::solve(&node_coordinates, &weights, None);
  let brute_sol = brute_algo::solve(&node_coordinates, &weights, None, thread_pool);

  let jeff_sol_len = compute_dist(&weights, &jeff_sol);
  let brute_sol_len = compute_dist(&weights, &brute_sol);

  let distance_diff = jeff_sol_len - brute_sol_len;

  if distance_diff.abs() > fp_epsilon && !is_identical_path(&jeff_sol, &brute_sol) { // account for floating point errors
    // re-do test, saving results
    let r_test_num: usize = rand::thread_rng().gen_range(0, 10000000);

    let prefix_dir = format!("./views/{:02}-{}/", weights.len(), r_test_num);
    jeff_algo::solve(&node_coordinates, &weights, Some(prefix_dir.clone()));
    brute_algo::solve_all(&node_coordinates, &weights, Some(prefix_dir.clone()), thread_pool);
    return false;
  }
  return true;
}

fn print_path_metadata(path: &Vec<usize>, weights: &Vec<Vec<fp>>) {
  println!("Solution distance: {}", compute_dist(weights, &path));
  print!("Solution order: ");
  for p in path {
    print!("{} ", *p);
  }
  println!("");
}

// Bounds some number i within len, used heavily in index calculations
fn b(i: usize, len: usize) -> usize {
  return (i + len) % len;
}

fn compute_dist(weights: &Vec<Vec<fp>>, path: &[usize]) -> fp {
  let mut total: fp = 0.0;
  for p_i in 0..path.len() {
    unsafe {
      let p  = path.get_unchecked(  p_i  );
      let p2 = path.get_unchecked(  (p_i+1) % path.len()  ); // mod lets us wrap at end (p_i == len(), (p_i+1) % len == 0)
      total += weights.get_unchecked( *p ).get_unchecked( *p2 );
    }
  }
  return total;
}

fn gen_tsp_problem(num_points: usize, min_x: fp, max_x: fp, min_y: fp, max_y: fp) -> (Vec<(usize, fp, fp)>, Vec<Vec<fp>>) {
  let mut rng = rand::thread_rng();
  let mut node_coordinates: Vec<(usize, fp, fp)> = vec![];

  for i in 0..num_points {
    node_coordinates.push(
      (i, rng.gen_range(min_x, max_x), rng.gen_range(min_y, max_y))
    );
  }

  // Compute 2x matrix of edge weights (assumes 2d euclidian geometry)
  let mut weights: Vec<Vec<fp>> = Vec::with_capacity(node_coordinates.len());
  {
    for row_r in &node_coordinates {
      let mut row_weight_v: Vec<fp> = Vec::with_capacity(node_coordinates.len());
      for col_r in &node_coordinates {
        let weight: fp = (
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

fn open_tsp_problem(file_arg: String) -> Option<(Vec<(usize, fp, fp)>, Vec<Vec<fp>>)> {
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

  let node_coordinates: Vec<(usize, fp, fp)> = node_coordinates.iter().map(|(a, b, c)| (*a, *b as fp, *c as fp) ).collect();

  // Compute 2x matrix of edge weights (assumes 2d euclidian geometry)
  let weights = compute_weight_coords(&node_coordinates);

  println!("City has {} points", weights.len());
  // remember weights is 2d square matrix (could be triangle, meh.)

  return Some( (node_coordinates, weights) );
}

// Meh used in imagery

fn compute_center(path: &Vec<usize>, locations: &Vec<(usize, fp, fp)>) -> (fp, fp) {
  let mut x_tot: fp = 0.0;
  let mut y_tot: fp = 0.0;

  for p in path {
    x_tot += locations[*p].1;
    y_tot += locations[*p].2;
  }

  x_tot /= path.len() as fp;
  y_tot /= path.len() as fp;
  return (x_tot, y_tot);
}

// Shared imagery functions

fn save_state_image<I: Into<String>>(file_path: I, path: &Vec<usize>, locations: &Vec<(usize, fp, fp)>) {
  let file_path = file_path.into();
  let (width, height) = (900, 900);
  let mut image = RgbImage::new(width + 15, height + 15); // width, height

  let (mut smallest_x, mut largest_y, mut largest_x, mut smallest_y) = get_point_extents(locations);

  if smallest_x > 0.0 {
    smallest_x = 0.0;
  }
  if largest_y < 15.0 {
    largest_y = 15.0;
  }
  if largest_x < 15.0 {
    largest_x = 15.0;
  }
  if smallest_y > 0.0 {
    smallest_y = 0.0;
  }
  // we'll expand to a 0x15 grid to normalize most of our data; if something larger comes in
  // the image will merely be skewed between runs by the furthest points

  // smallest_x -= 3.5;
  // largest_y += 3.5;
  // largest_x += 3.5;
  // smallest_y -= 3.5;

  let x_range: fp = largest_x - smallest_x;
  let y_range: fp = largest_y - smallest_y;

  let font = Font::try_from_bytes(include_bytes!("../resources/NotoSans-Bold.ttf")).unwrap();

  for i in 0..locations.len() {
    let loc = locations[i];
    let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.1, loc.2);

    // Set all location pixels to be red // r,g,b
    //image.get_pixel_mut(loc_x, loc_y).data = [255, 0, 0];
    //circle_it(&mut image, loc_x, loc_y, [255, 0, 0]);
    draw_hollow_circle_mut(&mut image, (loc_x as i32, loc_y as i32), 10 /*radius*/, Rgb([255, 0, 0]));

    // Also draw an index number
    let font_height = 18.0;
    let font_scale = Scale { x: font_height, y: font_height };
    draw_text_mut(&mut image, Rgb([225, 225, 255]), loc_x as u32, loc_y as u32, font_scale, &font, format!("{}", i).as_str());
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

  // does the folder exist?
  let file_parent_dir = std::path::PathBuf::from(file_path.clone());
  let file_parent_dir = file_parent_dir.parent().expect("All image paths should have a parent");
  std::fs::create_dir_all(&file_parent_dir).unwrap_or(());

  image.save(file_path).unwrap();
}

fn save_state_image_center<I: Into<String>>(file_path: I, path: &Vec<usize>, locations: &Vec<(usize, fp, fp)>, center: &(fp, fp)) {
  let file_path = file_path.into();
  let (width, height) = (600, 600);
  let mut image = RgbImage::new(width + 5, height + 5); // width, height

  let (smallest_x, largest_y, largest_x, smallest_y) = get_point_extents(locations);
  let x_range: fp = largest_x - smallest_x;
  let y_range: fp = largest_y - smallest_y;

  let font = Font::try_from_bytes(include_bytes!("../resources/NotoSans-Bold.ttf")).unwrap();

  for i in 0..locations.len() {
    let loc = locations[i];
    let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.1, loc.2);

    // Set all location pixels to be red // r,g,b
    //image.get_pixel_mut(loc_x, loc_y).data = [255, 0, 0];
    //circle_it(&mut image, loc_x, loc_y, [255, 0, 0]);
    draw_hollow_circle_mut(&mut image, (loc_x as i32, loc_y as i32), 10 /*radius*/, Rgb([255, 0, 0]));

    // Also draw an index number
    let font_height = 14.0;
    let font_scale = Scale { x: font_height, y: font_height };
    draw_text_mut(&mut image, Rgb([225, 225, 255]), loc_x as u32, loc_y as u32, font_scale, &font, format!("{}", i).as_str());
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

fn scale_xy(img_w: u32, img_h: u32, path_w: u32, path_h: u32, path_x_smallest: fp, path_y_smallest: fp, given_x: fp, given_y: fp) -> (u32, u32) {
  let mut img_x = (given_x - path_x_smallest) * ((img_w as fp / path_w as fp) as fp);
  let mut img_y = (given_y - path_y_smallest) * ((img_h as fp / path_h as fp) as fp);
  if img_x < 5.0 {
    img_x = 5.0;
  }
  if img_x > (img_w-5) as fp {
    img_x = (img_w-5) as fp;
  }
  if img_y < 5.0 {
    img_y = 5.0;
  }
  if img_y > (img_h-5) as fp {
    img_y = (img_h-5) as fp;
  }
  return (img_x as u32, img_y as u32);
}

// returns smallestX, largestY, largestX, smallestY
fn get_point_extents(locations: &Vec<(usize, fp, fp)>) -> (fp, fp, fp, fp) {
  let mut smallest_x = fp::INFINITY;
  let mut largest_y = fp::NEG_INFINITY;
  let mut largest_x = fp::NEG_INFINITY;
  let mut smallest_y = fp::INFINITY;
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

fn compute_weight_coords(node_coordinates: &Vec<(usize, fp, fp)>) -> Vec<Vec<fp>> {
  // Compute 2x matrix of edge weights (assumes 2d euclidian geometry)
  let mut weights: Vec<Vec<fp>> = Vec::with_capacity(node_coordinates.len());
  {
    for row_r in node_coordinates {
      let mut row_weight_v: Vec<fp> = Vec::with_capacity(node_coordinates.len());
      for col_r in node_coordinates {
        let weight: fp = (
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

fn selective(min_cities_to_ignore: usize, max_cities_to_test: usize, thread_pool: &ThreadPool, gpu_adapter: &mut Option<wgpu::Adapter>,) {
  println!("Performing selective failure from {} points to {} points...", min_cities_to_ignore, max_cities_to_test);
  // Bounding box for all points

  let mut rng = rand::thread_rng();
  let mut node_coordinates: Vec<(usize, fp, fp)> = vec![];

  // Just add 3 to begin with
  for i in 0..3 {
    let new_r_city = (
      i,
      rng.gen_range(x_min, x_max),
      rng.gen_range(y_min, y_max),
    );
    node_coordinates.push(new_r_city);
  }

  // Don't bother solving short tours which are highly likely to be correctly solved by jeff_algo
  // We can always manually configure this parameter down if this discoveres a horrible failure and
  // we want to find the first city where stuff blows up.
  for city_num in 3..min_cities_to_ignore {
    let new_r_city = (
      city_num,
      rng.gen_range(x_min, x_max),
      rng.gen_range(y_min, y_max),
    );
    node_coordinates.push(new_r_city);
  }

  // If we hit 11 cities without a failure we'll recurse and start from min_cities_to_ignore again.
  for city_num in min_cities_to_ignore..max_cities_to_test {
    let new_r_city = (
      city_num,
      rng.gen_range(x_min, x_max),
      rng.gen_range(y_min, y_max),
    );
    node_coordinates.push(new_r_city); // we can pop() if we fail

    let city_weights = compute_weight_coords(&node_coordinates);

    let jeff_sol = jeff_algo::solve(&node_coordinates, &city_weights, None);
    let brute_sol = brute_algo::solve(&node_coordinates, &city_weights, None, thread_pool);

    let jeff_sol_len = compute_dist(&city_weights, &jeff_sol);
    let brute_sol_len = compute_dist(&city_weights, &brute_sol);
    let distance_diff = jeff_sol_len - brute_sol_len;

    if distance_diff.abs() > fp_epsilon && !is_identical_path(&jeff_sol, &brute_sol) { // account for floating point errors
      println!("We have broken jeff_algo at {} points!", city_num+1);
      // we have added a city which breaks things!
      node_coordinates.pop();
      let city_weights = compute_weight_coords(&node_coordinates);

      // Now we have a city right before our failure.

      // Save the correct solution
      brute_algo::solve_all(&node_coordinates, &city_weights, Some("./views/selective/".to_string()), thread_pool);
      jeff_algo::solve(&node_coordinates, &city_weights, Some("./views/selective/".to_string()));

      // compute a 2d matrix of points and plot blue if they result in correct, red if they do not.
      // perform_matrix_image_gen("./views/selective-map.png", node_coordinates, city_weights, );

      // UPDATE: this is now done by spray() as a separate step.

      return;
    }
  }

  println!("Failed to break after {}, resetting...", max_cities_to_test);
  selective(min_cities_to_ignore, max_cities_to_test, thread_pool, gpu_adapter);

}

fn is_identical_path(path_a: &[usize], path_b: &[usize]) -> bool {
  if path_a.len() != path_b.len() {
    return false; // duh
  }

  let mut smallest_usize_in_a = usize::MAX;
  let mut smallest_usize_idx_in_a = 0;
  for i in 0..path_a.len() {
    if path_a[i] < smallest_usize_in_a {
      smallest_usize_in_a = path_a[i];
      smallest_usize_idx_in_a = i;
    }
  }

  let mut smallest_usize_idx_in_b = 0;
  for i in 0..path_b.len() {
    if path_b[i] == smallest_usize_in_a {
      smallest_usize_idx_in_b = i;
      break;
    }
  }

  // Both lists now have a begin index at their smallest value (assume 0)
  // we walk them & compare values; if any are not equal then these have
  // different orders!
  for i in 0..path_a.len() {
    if path_a[(i+smallest_usize_idx_in_a) % path_a.len()] != path_b[(i+smallest_usize_idx_in_b) % path_b.len()] {
      return false; // Not identical b/c values differ!
    }
  }

  return true; // identical b/c all path_a[i+] == path_b[i+]
}

fn get_env_or_random_node_coordinates(n: usize, env_var_name: &str, _x_min: fp, _x_max: fp, _y_min: fp, _y_max: fp) -> Vec<(usize, fp, fp)> {
  let mut rng = rand::thread_rng();
  let mut node_coordinates: Vec<(usize, fp, fp)> = vec![];
  // Create random set of points OR parse from env var
  match env::var(env_var_name) {
    Ok(initial_coords_s) => {
      // initial_coords_s == "5.12,6.8 4.8,4.9, 1.2,1.3"
      let pairs: Vec<&str> = initial_coords_s.split(" ").collect();
      for i in 0..n {
        let x_and_y_s: Vec<&str> = pairs[i].split(",").collect();
        let x: fp = x_and_y_s[0].parse().expect("TSP_INITIAL_COORDS did not contain a number");
        let y: fp = x_and_y_s[1].parse().expect("TSP_INITIAL_COORDS did not contain a number");
        let new_r_city = (
          i, x, y
        );
        node_coordinates.push(new_r_city);
      }
    }
    Err(_) => {
      for i in 0..n {
        let new_r_city = (
          i,
          rng.gen_range(_x_min, _x_max),
          rng.gen_range(_y_min, _y_max),
        );
        node_coordinates.push(new_r_city);
      }
    }
  }
  return node_coordinates;
}

fn spray(n: usize, mut bound_granularity: fp, thread_pool: &ThreadPool, gpu_adapter: &mut Option<wgpu::Adapter>,) {
  println!("Spraying {} cities...", n);

  if bound_granularity < 0.025 {
    println!("Resetting {} to 0.025 because that's the size of a single pixel...", bound_granularity);
    bound_granularity = 0.025;
  }
  let bound_granularity = bound_granularity;


  let node_coordinates: Vec<(usize, fp, fp)> = get_env_or_random_node_coordinates(n, "TSP_INITIAL_COORDS", x_min, x_max, y_min, y_max);
  println!("Initial node_coordinates={:?}", &node_coordinates);

  // Generate partial image
  let file_path = "views/spray.png";
  let (width, height) = (900, 900);
  let mut image = RgbImage::new(width + 15, height + 15); // width, height

  let (smallest_x, largest_y, largest_x, smallest_y) = (x_min_bound, y_max_bound, x_max_bound, y_min_bound);
  let x_range: fp = largest_x - smallest_x;
  let y_range: fp = largest_y - smallest_y;

  // Use jalgo to compute the first N-1 insertions...
  let city_weights = compute_weight_coords(&node_coordinates);
  let first_ordered_visits = jeff_algo::solve(&node_coordinates, &city_weights, None);

  let brute_sol = brute_algo::solve(&node_coordinates, &city_weights, None, thread_pool);
  // If jeff disagrees w/ brute, the rest of the loop does not make sense!
  let first_ordered_visits_len = compute_dist(&city_weights, &first_ordered_visits);
  let brute_sol_len = compute_dist(&city_weights, &brute_sol);
  let distance_diff = first_ordered_visits_len - brute_sol_len;
  if distance_diff.abs() > fp_epsilon && !is_identical_path(&first_ordered_visits, &brute_sol) {
    println!("Refusing to spray; jeff_sol={:?} ({}) and brute_sol={:?} ({}) are already broken!",
      first_ordered_visits, first_ordered_visits_len, brute_sol, brute_sol_len
    );
    return;
  }

  // Now test a grid of points every bound_granularity units,
  // computing the ideal and jalgo. When the two do not match, make a dot on
  // the spray image we generate.

  let mut num_failures = 0;

  let mut point_y = y_min_bound;
  loop {
    if point_y > y_max_bound {
      break;
    }

    let mut point_x = x_min_bound;
    loop {
      if point_x > x_max_bound {
        break;
      }

      let mut node_coordinates = node_coordinates.clone(); // Prevent us from mutating the initial set of points
      node_coordinates.push(
        (node_coordinates.len(), point_x, point_y)
      );
      // Now add (point_x, point_y) and see if it breaks jalgo

      let city_weights = compute_weight_coords(&node_coordinates);

      //let jeff_sol = jeff_algo::solve(&node_coordinates, &city_weights, None);
      //println!("=============");
      //let jeff_sol = jeff_algo::next_step(&first_ordered_visits, &node_coordinates, &city_weights, &None);
      let jeff_sol = jeff_algo::solve(&node_coordinates, &city_weights, None);
      //println!("jeff_sol={:?}", &jeff_sol);

      let brute_sol = brute_algo::solve(&node_coordinates, &city_weights, None, thread_pool);

      let jeff_sol_len = compute_dist(&city_weights, &jeff_sol);
      let brute_sol_len = compute_dist(&city_weights, &brute_sol);
      let distance_diff = jeff_sol_len - brute_sol_len;
      //println!("jeff_sol_len={}   brute_sol_len={}  distance_diff={}", jeff_sol_len, brute_sol_len, distance_diff);

      let loc = (point_x, point_y);
      let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.0, loc.1);

      if distance_diff.abs() > fp_epsilon && !is_identical_path(&jeff_sol, &brute_sol) {
        // jalgo broke, paint red pixel
        *image.get_pixel_mut(loc_x, loc_y) = Rgb([255, 0, 0]);
        {
          *image.get_pixel_mut(loc_x+1, loc_y) = Rgb([255, 0, 0]);
          *image.get_pixel_mut(loc_x+1, loc_y+1) = Rgb([255, 0, 0]);
          *image.get_pixel_mut(loc_x, loc_y+1) = Rgb([255, 0, 0]);
        }
        num_failures += 1;
        // Also save a copy of the state in views/spray-jalgo*
        // BUT only if bound_granularity > 0.1 as a performance improvement to high-res sprays
        if bound_granularity >= 0.2 {
          let prefix_dir = format!("./views/spray-jalgo-f{:03}", num_failures);

          // Debugging jeff_algo::next_step(&first_ordered_visits, &node_coordinates, &city_weights, &Some(format!("{}-jeff-next_step", prefix_dir.clone() ) ));
          //jeff_algo::solve(&node_coordinates, &city_weights, Some(prefix_dir.clone()));

          //brute_algo::solve(&node_coordinates, &city_weights, Some(prefix_dir.clone()));
          // Also dump brute_algo solutions for node_coordinates N-1, n-2, etc... until 3

          for i in 3..(node_coordinates.len()+1) {
            let mut delta_node_coords = vec![];
            for j in 0..i {
              delta_node_coords.push( node_coordinates[j] );
            }
            let city_weights = compute_weight_coords(&delta_node_coords);
            jeff_algo::solve(&delta_node_coords, &city_weights, Some(prefix_dir.clone()));
            brute_algo::solve_all(&delta_node_coords, &city_weights, Some(prefix_dir.clone()), thread_pool);
          }

        }
      }
      else {
        // jalgo got it correct, paint green
        *image.get_pixel_mut(loc_x, loc_y) = Rgb([0, 255, 0]);
      }

      point_x += bound_granularity;
    }

    point_y += bound_granularity;
  }

  let font = Font::try_from_bytes(include_bytes!("../resources/NotoSans-Bold.ttf")).unwrap();

  for i in 0..node_coordinates.len() {
    let loc = node_coordinates[i];
    let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.1, loc.2);

    // Set all location pixels to be red // r,g,b
    //image.get_pixel_mut(loc_x, loc_y).data = [255, 0, 0];
    //circle_it(&mut image, loc_x, loc_y, [255, 0, 0]);
    draw_hollow_circle_mut(&mut image, (loc_x as i32, loc_y as i32), 10 /*radius*/, Rgb([255, 0, 0]));

    // Also draw an index number

    let font_height = 18.0;
    let font_scale = Scale { x: font_height, y: font_height };
    draw_text_mut(&mut image, Rgb([225, 225, 255]), loc_x as u32, loc_y as u32, font_scale, &font, format!("{}", i).as_str());
  }

  // Finally write image to views/spray.png
  if let Err(e) = image.save(file_path) {
    println!("Please create the directory ./views/ before running tests!");
  }

  println!("{} failures", num_failures);

}

fn pattern_scan(n: usize, bound_granularity: fp, file_path: &str, thread_pool: &ThreadPool, gpu_adapter: &mut Option<wgpu::Adapter>) {
  let node_coordinates: Vec<(usize, fp, fp)> = get_env_or_random_node_coordinates(n, "TSP_INITIAL_COORDS", x_min, x_max, y_min, y_max);
  pattern_scan_coords(n, bound_granularity, file_path, node_coordinates, thread_pool, gpu_adapter, nop_closure);
}


//fn nop_closure() { } // apparenty Option<<Fn() -> ()>>::None is annoying to construct as a type
fn nop_closure(a: &Vec<Vec<CityWeight>>, b: &Vec<CityNum>, c: &(fp, fp), d: &(u8, u8, u8)) { }

fn pattern_scan_coords<F>(
  n: usize,
  mut bound_granularity: fp,
  file_path: &str,
  node_coordinates: Vec<(usize, fp, fp)>,
  thread_pool: &ThreadPool,
  gpu_adapter: &mut Option<wgpu::Adapter>,
  mut addtl_logging_fn: F,
) -> ()
  where F: std::ops::FnMut(&Vec<Vec<CityWeight>>, &Vec<CityNum>, &(fp, fp), &(u8, u8, u8)) -> (),
{
  println!("Pattern scanning {} cities...", n);
  if bound_granularity < 0.010 {
    println!("Resetting {} to 0.010 because that's the size of a single pixel...", bound_granularity);
    bound_granularity = 0.010;
  }
  let bound_granularity = bound_granularity;

  println!("Initial node_coordinates={:?}", &node_coordinates);

  // Generate partial image
  // let file_path = "views/pattern-scan.png";
  let (width, height) = (900, 900);
  let mut image = RgbImage::new(width + 15, height + 15); // width, height

  let (smallest_x, largest_y, largest_x, smallest_y) = (x_min_bound, y_max_bound, x_max_bound, y_min_bound);
  let x_range: fp = largest_x - smallest_x;
  let y_range: fp = largest_y - smallest_y;

  let city_weights = compute_weight_coords(&node_coordinates);

  let mut unique_solution_spaces_points: HashMap<(u8, u8, u8), Vec<(fp, fp)>> = HashMap::new();

  // If we get >1 brute solutions, pick next in line % all.
  // This is more deterministic than picking at random and produces a noticable checker pattern
  // on areas with 2+ solutions due to weight symmetry
  let mut brute_sol_nonce = 0;

  // If you get stripes instead of checkers, toggle the value of INCREMENT_NONCE_ON_ROW env variable to get the other pattern.
  let increment_nonce_on_row = env::var("INCREMENT_NONCE_ON_ROW").unwrap_or("f".to_string()).contains("t");

  let mut point_y = y_min_bound;
  loop {
    if point_y > y_max_bound {
      break;
    }

    if increment_nonce_on_row {
      brute_sol_nonce += 1; // bump so exactly-two are staggered at each row
    }

    let mut point_x = x_min_bound;
    loop {
      if point_x > x_max_bound {
        break;
      }

      let mut node_coordinates = node_coordinates.clone(); // Prevent us from mutating the initial set of points
      node_coordinates.push(
        (node_coordinates.len(), point_x, point_y)
      );
      // Now add (point_x, point_y) and see if it breaks jalgo

      let city_weights = compute_weight_coords(&node_coordinates);

      let brute_solutions = brute_algo::solve_all(&node_coordinates, &city_weights, None, thread_pool);
      let num_sols: i32 = brute_solutions.len() as i32;
      //let rand_idx: i32 = rand::thread_rng().gen_range(0, num_sols);
      //let brute_sol: Vec<CityNum> = brute_solutions[ rand_idx as usize ].clone(); // Vec<CityNum>
      let picked_idx: i32 = (brute_sol_nonce % brute_solutions.len()) as i32;
      let brute_sol: Vec<CityNum> = brute_solutions[ picked_idx as usize ].clone(); // Vec<CityNum>
      brute_sol_nonce += 1;

      let loc = (point_x, point_y);
      let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.0, loc.1);

      // Paint according to brute_sol order
      let (r, g, b) = path_to_rgb(&brute_sol, &city_weights);

      let rgb_key = (r, g, b);
      if !unique_solution_spaces_points.contains_key(&rgb_key) {
        unique_solution_spaces_points.insert(rgb_key, vec![]);
      }
      unique_solution_spaces_points.get_mut(&rgb_key).map(|key_vec| { key_vec.push( (point_x, point_y) ); });

      addtl_logging_fn(
        &city_weights,
        &brute_sol,
        &(point_x, point_y),
        &rgb_key
      );

      // println!("RGB of {:?} is {}, {}, {}", brute_sol, r, g, b);

      //*image.get_pixel_mut(loc_x, loc_y) = Rgb([r, g, b]);

      // Larger 4x4 dots
      *image.get_pixel_mut(loc_x, loc_y) = Rgb([r, g, b]);
      {
        *image.get_pixel_mut(loc_x+1, loc_y) = Rgb([r, g, b]);
        *image.get_pixel_mut(loc_x+1, loc_y+1) = Rgb([r, g, b]);
        *image.get_pixel_mut(loc_x, loc_y+1) = Rgb([r, g, b]);
      }

      point_x += bound_granularity;
    }

    point_y += bound_granularity;
  }

  let font = Font::try_from_bytes(include_bytes!("../resources/NotoSans-Bold.ttf")).unwrap();

  for i in 0..node_coordinates.len() {
    let loc = node_coordinates[i];
    let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, loc.1, loc.2);

    // Set all location pixels to be red // r,g,b
    //image.get_pixel_mut(loc_x, loc_y).data = [255, 0, 0];
    //circle_it(&mut image, loc_x, loc_y, [255, 0, 0]);
    draw_hollow_circle_mut(&mut image, (loc_x as i32, loc_y as i32), 10 /*radius*/, Rgb([255, 0, 0]));

    // Also draw an index number

    let font_height = 18.0;
    let font_scale = Scale { x: font_height, y: font_height };
    draw_text_mut(&mut image, Rgb([225, 225, 255]), loc_x as u32, loc_y as u32, font_scale, &font, format!("{}", i).as_str());
  }

  // Get average of all points over unique_solution_spaces_points
  // and draw / log brute path for the average point of a shared collection of solutions
  let file_path_name = std::path::Path::new(file_path).file_name().unwrap();
  let file_path_name = &file_path_name.to_str().unwrap();
  let file_path_name = file_path_name.replace(".png", "").replace(".jpg", "");

  let mut space_label_y_coords = vec![];

  for (rgb_key, inserted_points) in unique_solution_spaces_points.iter() {
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for (x, y) in inserted_points.iter() {
      sum_x += x;
      sum_y += y;
    }
    let avg_x = sum_x / (inserted_points.len() as fp);
    let avg_y = sum_y / (inserted_points.len() as fp);

    // Draw text
    let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, avg_x, avg_y);
    let rgb_text = format!("{:02x}{:02x}{:02x}", rgb_key.0, rgb_key.1, rgb_key.2 );
    let font_height = 18.0;
    let font_scale = Scale { x: font_height, y: font_height };

    // Apply a random += y delta for the label to prevent labels from overlapping in outputs
    //let loc_y = ( loc_y as i32 + ( rand::thread_rng().gen_range(-32, 32) as i32 ) ) as u32;

    // Is loc_y within 18px of anything in space_label_y_coords?
    // If so increment by 6px until no longer overlapping anything
    let mut loc_y = loc_y - 46;
    loop {
      let mut y_is_overlapping = false;

      for existing_y_coord in space_label_y_coords.iter() {
        let existing_y_coord_min = existing_y_coord - 16;
        let existing_y_coord_max = existing_y_coord + 16;
        if loc_y > existing_y_coord_min && loc_y < existing_y_coord_max {
          y_is_overlapping = true;
        }
      }

      if y_is_overlapping {
        loc_y += 6;
      }
      else {
        break; // done!
      }
    }

    space_label_y_coords.push(loc_y as u32);
    draw_text_mut(&mut image, Rgb([225, 225, 255]), loc_x as u32, loc_y as u32, font_scale, &font, rgb_text.as_str());

    let mut node_coordinates = node_coordinates.clone(); // Prevent us from mutating the initial set of points
    node_coordinates.push(
      (node_coordinates.len(), avg_x, avg_y)
    );

    // Also compute brute force for avg_x, avg_y and store under views/pattern-scan-{rgb_text}-center/

    let parent_prefix_dir = format!("views/{}", file_path_name);
    std::fs::create_dir_all(&parent_prefix_dir).unwrap_or(());

    let prefix_dir = format!("views/{}/{}-center", file_path_name, rgb_text);
    for i in 3..(node_coordinates.len()+1) {
      let mut delta_node_coords = vec![];
      for j in 0..i {
        delta_node_coords.push( node_coordinates[j] );
      }
      let city_weights = compute_weight_coords(&delta_node_coords);
      brute_algo::solve_all(&delta_node_coords, &city_weights, Some(prefix_dir.clone()), thread_pool);
    }

  }



  // Finally write image to views/pattern_scan.png
  if let Err(e) = image.save(file_path) {
    println!("Please create the directory ./views/ before running tests!");
  }

  println!("{} unique solutions found + plotted!", unique_solution_spaces_points.len());

}

fn multi_pattern_scan(n: usize, bound_granularity: fp, num_multi_steps_to_scan: usize, thread_pool: &ThreadPool, gpu_adapter: &mut Option<wgpu::Adapter>,) {
  println!("Muti-pattern scanning {} cities...", n);

  let node_coordinates_a: Vec<(usize, fp, fp)> = get_env_or_random_node_coordinates(n, "TSP_INITIAL_COORDS", x_min, x_max, y_min, y_max);
  println!("Initial node_coordinates_a={:?}", &node_coordinates_a);

  let node_coordinates_b: Vec<(usize, fp, fp)> = get_env_or_random_node_coordinates(n, "TSP_ENDING_COORDS", x_min, x_max, y_min, y_max);
  println!("Initial node_coordinates_b={:?}", &node_coordinates_b);

  let mut output_scan_files = vec![];
  let mut output_multiscan_parabola_file_paths = vec![];

  for multi_step_i in 0..=num_multi_steps_to_scan {
    let converged_cities = converge_coordinates(&node_coordinates_a, &node_coordinates_b, multi_step_i, num_multi_steps_to_scan);
    let converged_cities_weights = compute_weight_coords(&converged_cities);
    let initial_solution = brute_algo::solve(&converged_cities, &converged_cities_weights, None, thread_pool);
    let output_multiscan_file_path = format!("views/multi-pattern-scan-{:03}.png", multi_step_i);
    let html_path = format!("views/multi-pattern-scan-{:03}.html", multi_step_i);
    let mut html_content = HTML_BEGIN.to_string();

    // [(x, y, rgb_usize, ) ... ]
    let mut tsp_point_colors: Vec<(fp, fp, usize)> = vec![];

    pattern_scan_coords(n, bound_granularity, &output_multiscan_file_path, converged_cities.clone(), thread_pool, gpu_adapter, |city_weights, brute_sol, (tsp_point_x, tsp_point_y), rgb_key| {
      let point_x: isize = (tsp_point_x * HTML_POINT_SCALE) as isize;
      let point_y: isize = (tsp_point_y * HTML_POINT_SCALE) as isize;

      let mut c = "".to_string();
      for city_num in brute_sol.iter() {
        if *city_num < converged_cities.len() {
          c += format!("{},{} ", (converged_cities[*city_num].1 * HTML_POINT_SCALE) as isize, (converged_cities[*city_num].2 * HTML_POINT_SCALE) as isize).as_str();
        }
        else { // it's the new one
          c += format!("{},{} ", point_x, point_y).as_str();
        }
      }

      let city_weights = normalize_weights(city_weights);
      html_content += format!(
        "<div style=\"width:5px;height:5px;position:absolute;left:{}px;top:{}px;background-color:#{:02x}{:02x}{:02x}\" onclick=\"draw_path(this);\" c=\"{}\">{}</div>",
        point_x,point_y,
        rgb_key.0, rgb_key.1, rgb_key.2,
        c, // coordinated mapped to display coords, ought to match everything else
        html_format_tour_details(&city_weights, brute_sol).as_str()
      ).as_str();

      let rgb_key: usize = ((rgb_key.0 as usize) << 16) + ((rgb_key.1 as usize) << 8) + (rgb_key.2 as usize);
      tsp_point_colors.push(
        (*tsp_point_x, *tsp_point_y, rgb_key)
      );


    });

    let output_multiscan_parabola_file_path = format!("views/multi-pattern-scan-{:03}-parabola.png", multi_step_i);
    let output_multiscan_parabola_txt_file_path = format!("views/multi-pattern-scan-{:03}-parabola.txt", multi_step_i);
    // Edge detection w/ tsp_points_colors
    let mut parabola_points: HashMap<usize, Vec<(fp, fp)>> = HashMap::new(); // RGB color string -> list of points on ... exterior.. hmm.

    // We know tsp_point_colors contains a square, so compute width & height so we can index into neighbors for edge detection
    let tsp_square_size: isize = (f64::sqrt(tsp_point_colors.len() as f64) as isize /*+ 1*/) as isize;

    for y in 0..tsp_square_size {
      for x in 0..tsp_square_size {
        let (tsp_point_x, tsp_point_y, rgb_key) = tsp_point_colors[((y * tsp_square_size) + x) as usize];

        // First question; are our neighbors rgb_key s different?

        // "y-minus-one-index", "x-plus-one-index", etc.
        let ym1i: isize = ((y-1) * tsp_square_size) + x;
        let yp1i: isize = ((y+1) * tsp_square_size) + x;
        let xm1i: isize = (y*tsp_square_size) + (x-1);
        let xp1i: isize = (y*tsp_square_size) + (x+1);

        if ym1i <= 0 || yp1i >= tsp_point_colors.len() as isize || xm1i <= 0 || xp1i >= tsp_point_colors.len() as isize  {
          continue; // off-square! (incl. borders to avoid comparing pixel colors against black in output files)
        }

        #[allow(unused_parens)]
        let is_edge_pt = (
          rgb_key != tsp_point_colors[ym1i as usize].2 ||  // is y-1 different color?
          rgb_key != tsp_point_colors[yp1i as usize].2 ||  // is y+1 different color?
          rgb_key != tsp_point_colors[xm1i as usize].2 ||  // is x-1 different color?
          rgb_key != tsp_point_colors[xp1i as usize].2     // is x+1 different color?
        );
        if !is_edge_pt {
          continue;
        }

        // Throw out the straight lines on square's border, we're not interested in those!
        if is_edge_pt && (x <= 2 || x >= tsp_square_size - 2 || y <= 2 || y >= tsp_square_size - 2 ) {
          continue;
        }

        if !parabola_points.contains_key(&rgb_key) {
          parabola_points.insert(rgb_key, vec![]);
        }

        if let Some(parabola_points_vec) = parabola_points.get_mut(&rgb_key) {
          parabola_points_vec.push(
            (tsp_point_x, tsp_point_y)
          );
        }

      }
    }

    // now use each set of key, list of parabola_points to predict N polynominals

    // Iterate all TSP edge points + store lists of continuous strings w/ different (a color, b color) sides;
    // these will be our parabolic edges which we can algebra functions out of?

    // List of A -> B MIDPOINTS; average x,y of two most-nearby points as we iterate forwards along both.
    // We expect each pair of lines to have within 1-2 number of the same points.
    // keys are (std::cmp::min(A, B), std::cmp::max(A, B)) so we keep both color data and collection order does not matter.
    let mut functions_edge_points: HashMap<(usize, usize), Vec<(fp, fp)>> = HashMap::new();

    let matching_radius = (bound_granularity.powf(2.0) + bound_granularity.powf(2.0)).sqrt() * 1.02; // sqrt(A**2 + B**2) plus 2% error to fetch points on diagonals

    for (rgb_key_a, edge_tsp_points_a) in &parabola_points {
      // for all points within bound_granularity of a point in edge_tsp_points_a,
      // take average and add to vec. If vec len > /*0*/ 2, add to functions_edge_points.
      let mut nearby_pts_colors: Vec<(fp, fp, usize)> = vec![];
      for (a_tsp_pt_x, a_tsp_pt_y) in edge_tsp_points_a {
        nearby_pts_colors.push(
          (*a_tsp_pt_x, *a_tsp_pt_y, *rgb_key_a) // include a data
        );
        for (rgb_key_b, edge_tsp_points_b) in &parabola_points {
          if *rgb_key_b == *rgb_key_a {
            continue;
          }
          for (b_tsp_pt_x, b_tsp_pt_y) in edge_tsp_points_b {
            let ab_pt_dist = ((a_tsp_pt_x - b_tsp_pt_x).powf(2.0) + (a_tsp_pt_y - b_tsp_pt_y).powf(2.0)).sqrt();
            if ab_pt_dist < matching_radius {
              // points are co-located, add b data!
              nearby_pts_colors.push(
                (*b_tsp_pt_x, *b_tsp_pt_y, *rgb_key_b)
              );
            }
          }
        }
      }

      let nearby_pts_colors = nearby_pts_colors;
      // Now we have all nearby_pts_colors for this edge_tsp_points_a;
      // Walk the original line again and insert nearby points to functions_edge_points
      // along their A-B vectors by taking the average of all nearby points.
      for (a_tsp_pt_x, a_tsp_pt_y) in edge_tsp_points_a {
        let mut other_rgb_key_b_val = rgb_key_a;
        let mut rgb_key_b_nearby_positions : Vec<(fp, fp)> = vec![];
        for (b_tsp_pt_x, b_tsp_pt_y, rgb_key_b) in &nearby_pts_colors {
          if rgb_key_b == rgb_key_a || (other_rgb_key_b_val != rgb_key_a && rgb_key_b != other_rgb_key_b_val ) {
            continue; // Skip our own x,y points AND not-the-first-value-of-rgb_key_b points.
          }
          // Are we within matching_radius of a_tsp_pt_x, a_tsp_pt_y?
          let ab_pt_dist = ((a_tsp_pt_x - b_tsp_pt_x).powf(2.0) + (a_tsp_pt_y - b_tsp_pt_y).powf(2.0)).sqrt();
          if ab_pt_dist < matching_radius {
            other_rgb_key_b_val = rgb_key_b; // First rgb_key_b wins, but we expect only 1 in 99% of cases so that's fine.
            rgb_key_b_nearby_positions.push(
              (*b_tsp_pt_x, *b_tsp_pt_y)
            );
          }
        }
        if other_rgb_key_b_val == rgb_key_a {
          continue; // apparently nearby_pts_colors.len() == 0, don't store empty data.
        }
        // We now guarantee that other_rgb_key_b_val is not rgb_key_a
        // Now average all points in rgb_key_b_nearby_positions and store in a->b the key
        let mut total_pt_x: fp = *a_tsp_pt_x;
        let mut total_pt_y: fp = *a_tsp_pt_y;
        for (b_tsp_pt_x, b_tsp_pt_y) in &rgb_key_b_nearby_positions {
          total_pt_x += *b_tsp_pt_x;
          total_pt_y += *b_tsp_pt_y;
        }

        let avg_pt_x: fp = total_pt_x / ((rgb_key_b_nearby_positions.len()+1) as fp);
        let avg_pt_y: fp = total_pt_y / ((rgb_key_b_nearby_positions.len()+1) as fp);

        let functions_edge_points_key: (usize, usize) = (
          std::cmp::min(*rgb_key_a, *other_rgb_key_b_val),
          std::cmp::max(*rgb_key_a, *other_rgb_key_b_val)
        );

        if ! functions_edge_points.contains_key(&functions_edge_points_key) {
          functions_edge_points.insert(functions_edge_points_key, vec![] );
        }

        if let Some(functions_edge_points_vec) = functions_edge_points.get_mut(&functions_edge_points_key) {
          functions_edge_points_vec.push(
            (avg_pt_x, avg_pt_y)
          );
        }

      }

    }

    // Now we can assume functions_edge_points is full;
    // compute Ax^2 + Bxy + Cy^2 + Dx + Ey + F == 0 coefficients and store in the map.
    let mut functions_edge_xy_abcdef_coef: HashMap<(usize, usize), (fp, fp, fp, fp, fp, fp /*A,B,C,D,E,F*/)> = HashMap::new();

    for (edge_keys, edge_points_vec) in &functions_edge_points {
      if edge_points_vec.len() < /*3*/ 6 {
        continue; // not enough data!
      }

      // Grab 6 equi-distant points to use in gomez to solve the general conics equations
      // to find A, B, C, D, E, and F for
      // Ax^2 + Bxy + Cy^2 + Dx + Ey + F == 0

      let one_sixth_dist = edge_points_vec.len() / 6;

      /*let (x1, y1) = edge_points_vec[0];
      let (x2, y2) = edge_points_vec[1 * one_sixth_dist];
      let (x3, y3) = edge_points_vec[2 * one_sixth_dist];
      let (x4, y4) = edge_points_vec[3 * one_sixth_dist];
      let (x5, y5) = edge_points_vec[4 * one_sixth_dist];
      let (x6, y6) = edge_points_vec[edge_points_vec.len() - 1];*/

      let fit_pts_json_file = format!("{}-{:x}-{:x}-edge-points.json", output_multiscan_parabola_txt_file_path, edge_keys.0, edge_keys.1);
      let mut fit_pts_json_txt = String::new(); //format!("{:?}", edge_points_vec);
      fit_pts_json_txt += "[";
      for (point_x, point_y) in edge_points_vec {
        fit_pts_json_txt += format!("[{point_x}, {point_y}], ").as_str();
      }
      fit_pts_json_txt.pop();
      fit_pts_json_txt.pop(); // Remove last ", " chars
      fit_pts_json_txt += "]";
      let mut f = File::create(&fit_pts_json_file).expect("Unable to create file");
      f.write_all(fit_pts_json_txt.as_bytes()).expect("Unable to write data");

      let (a, b, c, d, e, f) = parabolics::solve_for_6pts(
        thread_pool, gpu_adapter,
        edge_points_vec[0],
        edge_points_vec[1 * one_sixth_dist],
        edge_points_vec[2 * one_sixth_dist],
        edge_points_vec[3 * one_sixth_dist],
        edge_points_vec[4 * one_sixth_dist],
        edge_points_vec[edge_points_vec.len() - 1]
      );

      functions_edge_xy_abcdef_coef.insert(*edge_keys, (a, b, c, d, e, f));

    }


    { // text out
      let mut parabola_txt = String::new();
      for (rgb_key, edge_tsp_points) in &parabola_points {
        let num_points = edge_tsp_points.len();
        parabola_txt += format!("=== {rgb_key:x} ({num_points} points) ===\n").as_str();
        for (edge_pt_x, edge_pt_y) in edge_tsp_points {
          parabola_txt += format!("  {edge_pt_x}, {edge_pt_y}\n").as_str();
        }
        parabola_txt += "\n";
      }

      parabola_txt += "=== === === ===\n";
      parabola_txt += "===  EDGES  ===\n";
      parabola_txt += "=== === === ===\n";

      for (edge_keys, edge_points_vec) in &functions_edge_points {
        let (a,b,c,d,e,f) = functions_edge_xy_abcdef_coef.get(edge_keys).unwrap_or(&(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)); // we know it exists

        parabola_txt += format!("Edge {:06x} - {:06x} has {} points,  ({} * x**2) + ({} * xy) + ({} * y**2) + ({} * x) + ({} * y) + {} = 0 \n",
          edge_keys.0, edge_keys.1, edge_points_vec.len(), a,b,c,d,e,f
        ).as_str();

        for (edge_tsp_x, edge_tsp_y) in edge_points_vec {
          parabola_txt += format!("  {}, {}\n", edge_tsp_x, edge_tsp_y).as_str();
        }
        parabola_txt += "\n";
      }
      parabola_txt += "\n";

      let mut f = File::create(&output_multiscan_parabola_txt_file_path).expect("Unable to create file");
      f.write_all(parabola_txt.as_bytes()).expect("Unable to write data");
    }


    { // image out
      let (width, height) = (900, 900);
      let mut image = RgbImage::new(width + 15, height + 15); // width, height
      let font = Font::try_from_bytes(include_bytes!("../resources/NotoSans-Bold.ttf")).unwrap();

      let (smallest_x, largest_y, largest_x, smallest_y) = (x_min_bound, y_max_bound, x_max_bound, y_min_bound);
      let x_range: fp = largest_x - smallest_x;
      let y_range: fp = largest_y - smallest_y;

      let mut skip_nonce: usize = 0;
      for (rgb_key, edge_tsp_points) in &parabola_points {
        for (tsp_edge_point_x, tsp_edge_point_y) in edge_tsp_points {
          // todo draw an image into output_multiscan_parabola_file_path

          skip_nonce += 1;

          if skip_nonce % 6 != 0 { // 1/6 of the time put a pixel down - we want spaces to differentiate lines with
            continue;
          }

          let r: u8 = ((rgb_key >> 16) & 0xff) as u8;
          let g: u8 = ((rgb_key >>  8) & 0xff) as u8;
          let b: u8 = ((rgb_key >>  0) & 0xff) as u8;
          let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, *tsp_edge_point_x, *tsp_edge_point_y);

          *image.get_pixel_mut(loc_x, loc_y) = Rgb([r, g, b]);
          {
            *image.get_pixel_mut(loc_x+1, loc_y) = Rgb([r, g, b]);
            *image.get_pixel_mut(loc_x+1, loc_y+1) = Rgb([r, g, b]);
            *image.get_pixel_mut(loc_x, loc_y+1) = Rgb([r, g, b]);
          }

        }
      }

      let x_midpt = (smallest_x+largest_x)/2.0;
      let y_midpt = (smallest_y+largest_y)/2.0;

      // Also draw parabolas in white using functions_edge_xy_abcdef_coef
      for ((rgb_key_1, rgb_key_2), (a, b, c, d, e, f)) in &functions_edge_xy_abcdef_coef {

        // Edge colors is straight-up combo of the 2 component colors
        let col_r: u8 = ( (((rgb_key_1 >> 16) & 0xff)+((rgb_key_2 >> 16) & 0xff)) /2) as u8;
        let col_g: u8 = ( (((rgb_key_1 >> 8) & 0xff)+((rgb_key_2 >> 8) & 0xff)) /2) as u8;
        let col_b: u8 = ( (((rgb_key_1 >> 0) & 0xff)+((rgb_key_2 >> 0) & 0xff)) /2) as u8;

        // Draw in steps from smallest_x -> largest_x, keeping where Y falls into range.
        let mut x = smallest_x;
        let mut curve_total_x: fp = 0.0;
        let mut curve_total_y: fp = 0.0;
        let mut curve_num_pts: usize = 0;
        loop {
          if x >= largest_x {
            break;
          }

          for y in parabolics::evaluate_parabolic_for_x(x, (*a, *b, *c, *d, *e, *f)) {
            if y > smallest_y && y < largest_y {
              // Transform TSP x and y to image x and y and drop some ink on it!

              curve_total_x += x;
              curve_total_y += y;
              curve_num_pts += 1;

              //let r: u8 = 255;
              //let g: u8 = 255;
              //let b: u8 = 255;
              let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, x, y);

              *image.get_pixel_mut(loc_x, loc_y) = Rgb([col_r, col_g, col_b]);

            }
          }

          x += bound_granularity * 0.025; // 40x precision
        }

        let curve_avg_x = curve_total_x / curve_num_pts as fp;
        let curve_avg_y = curve_total_y / curve_num_pts as fp;
        let (loc_x,loc_y) = scale_xy(width, height, x_range as u32, y_range as u32, smallest_x, smallest_y, curve_avg_x, curve_avg_y);
        // Drop a label at average x,y for curve

        let label_txt = format!("{:06x} - {:06x}", rgb_key_1, rgb_key_2);

        let font_height = 18.0;
        let font_scale = Scale { x: font_height, y: font_height };
        draw_text_mut(&mut image, Rgb([col_r, col_g, col_b]), loc_x as u32, loc_y as u32, font_scale, &font, label_txt.as_str());

        let eq_txt = format!(
          "{}x^2 + {}xy + {}y^2 + {}x + {}y + {} = 0",
          a, b, c, d, e, f
        );
        draw_text_mut(&mut image, Rgb([col_r, col_g, col_b]), loc_x as u32, (loc_y-24) as u32, font_scale, &font, eq_txt.as_str());


      }

      image.save(output_multiscan_parabola_file_path.clone()).unwrap();

      output_multiscan_parabola_file_paths.push(output_multiscan_parabola_file_path.clone());
    }






    // Overlap tested N+1 points w/ beginning coordinates
    for (city_num, city_x, city_y) in converged_cities.iter() {
      let city_x: isize = (city_x * HTML_POINT_SCALE) as isize;
      let city_y: isize = (city_y * HTML_POINT_SCALE) as isize;
      html_content += format!(
        "<div style=\"width:24px;height:24px;position:absolute;left:{}px;top:{}px;background:transparent;border: 3px solid red;border-radius:99px;font-weight:bold;pointer-events:none;\">{}</div>",
        city_x - 12, city_y - 12, city_num
      ).as_str();
    }

    let mut c = "".to_string();
    for city_num in initial_solution.iter() {
      c += format!("{},{} ", (converged_cities[*city_num].1 * HTML_POINT_SCALE) as isize, (converged_cities[*city_num].2 * HTML_POINT_SCALE) as isize).as_str();
    }

    html_content += format!("<pre style=\"position:absolute;top:1600px;height:350px;\" id=\"initial-sol\" c=\"{}\">initial_solution = {:?}</pre>", c, &initial_solution).as_str();
    html_content += HTML_END;

    let mut f = File::create(&html_path).expect("Unable to create file");
    f.write_all(html_content.as_bytes()).expect("Unable to write data");

    output_scan_files.push(output_multiscan_file_path);
  }

  let gif_output_file = "views/multi-pattern-scan.gif";

  let images = engiffen::load_images(&output_scan_files);
  if let Ok(gif_data) = engiffen::engiffen(&images, 5 /*fps*/, engiffen::Quantizer::Naive ) {
    if let Ok(mut output_f) = File::create(gif_output_file) {
      if let Err(e) = gif_data.write(&mut output_f) {
        eprintln!("Error writing to {}: {:?}", gif_output_file, e);
      }
    }
  }
  println!("See {}", gif_output_file);


  // Parabola gif as well
  let gif_output_file = "views/multi-pattern-scan-parabola.gif";

  let images = engiffen::load_images(&output_multiscan_parabola_file_paths);
  if let Ok(gif_data) = engiffen::engiffen(&images, 5 /*fps*/, engiffen::Quantizer::Naive ) {
    if let Ok(mut output_f) = File::create(gif_output_file) {
      if let Err(e) = gif_data.write(&mut output_f) {
        eprintln!("Error writing to {}: {:?}", gif_output_file, e);
      }
    }
  }


}

fn converge_coordinates(a: &Vec<(usize, fp, fp)>, b: &Vec<(usize, fp, fp)>, step_num: usize, total_steps: usize) -> Vec<(usize, fp, fp)> {
  if a.len() != b.len() {
    panic!("a.len() = {} and b.len() = {}", a.len(), b.len());
  }
  let mut converged = vec![];
  for i in 0..a.len() {
    converged.push(
      converge_coords(a[i], b[i], step_num, total_steps)
    );
  }
  converged
}

fn converge_coords(a: (usize, fp, fp), b: (usize, fp, fp), step_num: usize, total_steps: usize) -> (usize, fp, fp) {
  let a_weight = (total_steps - step_num) as fp / total_steps as fp;
  let b_weight = step_num as fp / total_steps as fp;
  #[allow(non_upper_case_globals)]
  const rounded_decimals: fp = 1000.0;
  (
    a.0,
    (((a.1 * a_weight) + (b.1 * b_weight)) * rounded_decimals).round() / rounded_decimals,
    (((a.2 * a_weight) + (b.2 * b_weight)) * rounded_decimals).round() / rounded_decimals,
  )
}


static PATH_TO_RGB_CACHE: Lazy<Mutex<HashMap<usize, (u8, u8, u8) >>> = Lazy::new(|| {
  Mutex::new( HashMap::new() )
});

pub fn path_to_rgb(path: &[usize], city_weights: &Vec<Vec<fp>>) -> (u8, u8, u8) {

  // Iterate city from zero_i to end_i, calculating a hash in both directions.

  let mut zero_i = 0;
  for i in 0..path.len() {
    if path[i] == 0 {
      zero_i = i;
    }
  }
  let zero_i = zero_i;

  let end_i = (zero_i + (path.len() - 1) ) % path.len();

  let mut left_i = zero_i;
  let mut right_i = end_i;
  let mut left_hash = std::collections::hash_map::DefaultHasher::default();
  let mut right_hash = std::collections::hash_map::DefaultHasher::default();

  path[zero_i].hash(&mut right_hash); // right hash must visit 0 first to prevent off-by-one during flipped path comparisons.

  loop {
    // Hash the value into S, we only care about value and aligned ordering.
    path[left_i].hash(&mut left_hash);
    path[right_i].hash(&mut right_hash);

    left_i = (left_i+1) % path.len(); // increment w/ wrap
    right_i = (right_i+(path.len()-1)) % path.len(); // decrement w/ wrap

    if left_i == end_i { // if this is ever NOT the terminating case I'm fine w/ a loud process hang
      if right_i != zero_i {
        panic!("Invariant violation, expected when left_i={} == end_i that right_i ({}) == {}", left_i, right_i, zero_i);
      }
      path[left_i].hash(&mut left_hash);
      //path[right_i].hash(&mut right_hash); // cannot hash here, see instruction directly above loop{}
      break;
    }
  }
  let left_hash_u64 = left_hash.finish() as usize;
  let right_hash_u64 = right_hash.finish() as usize;

  // If hash_u64 is in cache, re-use same color.
  // Else generate something random but "nice" and store in cache.

  let mut path_to_rgb_cache_ref = PATH_TO_RGB_CACHE.lock().unwrap();
  if let Some(colors) = path_to_rgb_cache_ref.get(&left_hash_u64) {
    return *colors;
  }
  else if let Some(colors) = path_to_rgb_cache_ref.get(&right_hash_u64) {
    return *colors;
  }
  else {
    let r = rand::thread_rng().gen_range(40, 220) as u8;
    let g = rand::thread_rng().gen_range(40, 220) as u8;
    let b = rand::thread_rng().gen_range(40, 220) as u8;

    // for all future hash_u64s in THIS process, re-use same color.
    // Not deterministic across machines.
    path_to_rgb_cache_ref.insert(left_hash_u64, (r, g, b));
    path_to_rgb_cache_ref.insert(right_hash_u64, (r, g, b));

    // Debugging
    //println!("Unique color ({:02x}{:02x}{:02x}) allocated for path = {:?} hashes=({}, {})", r, g, b, path, left_hash_u64, right_hash_u64);

    return (r, g, b);
  }
}



fn spray_pattern_search(n: usize, bound_granularity: fp, num_sprays_to_perform: usize, thread_pool: &ThreadPool, gpu_adapter: &mut Option<wgpu::Adapter>,) {
  println!("Spray pattern searching {} cities for {} sprays...", n, num_sprays_to_perform);

  if brute_algo::use_brute_cache_env_val() {
    println!("Refusing to run w/ brute cache enabled, this will generate a ton of un-used entries. Set USE_BRUTE_CACHE=f before running");
    return;
  }

  for spray_i in 0..num_sprays_to_perform {
    // Generate random N-city
    let node_coordinates: Vec<(usize, fp, fp)> = get_env_or_random_node_coordinates(n, "SHOULD_NEVER_BE_COORDS_HERE!!__$%@!#@#!#&(*#@_INVALID_CHARS", x_min, x_max, y_min, y_max);

    println!("");
    println!("spray_i={:03} node_coordinates={:?}", spray_i, node_coordinates);

    let file_path = format!("views/spray-pattern-search-{:03}.png", spray_i);
    let html_path = format!("views/spray-pattern-search-{:03}.html", spray_i);
    let mut html_content = HTML_BEGIN.to_string();

    pattern_scan_coords(n, bound_granularity, &file_path, node_coordinates.clone(), thread_pool, gpu_adapter, |city_weights, brute_sol, (point_x, point_y), rgb_key| {
      let point_x: isize = (point_x * HTML_POINT_SCALE) as isize;
      let point_y: isize = (point_y * HTML_POINT_SCALE) as isize;

      let city_weights = normalize_weights(city_weights);
      html_content += format!(
        "<div style=\"width:5px;height:5px;position:absolute;left:{}px;top:{}px;background-color:#{:02x}{:02x}{:02x}\" onclick=\"draw_path(this);\">{}</div>",
        point_x,point_y,
        rgb_key.0, rgb_key.1, rgb_key.2,
        html_format_tour_details(&city_weights, brute_sol).as_str()
      ).as_str();

    });

    // Overlap tested N+1 points w/ beginning coordinates
    for (city_num, city_x, city_y) in node_coordinates.iter() {
      let city_x: isize = (city_x * HTML_POINT_SCALE) as isize;
      let city_y: isize = (city_y * HTML_POINT_SCALE) as isize;
      html_content += format!(
        "<div style=\"width:24px;height:24px;position:absolute;left:{}px;top:{}px;background:transparent;border: 3px solid red;border-radius:99px;font-weight:bold;pointer-events:none;\">{}</div>",
        city_x - 12, city_y - 12, city_num
      ).as_str();
    }

    html_content += HTML_END;

    let mut f = File::create(&html_path).expect("Unable to create file");
    f.write_all(html_content.as_bytes()).expect("Unable to write data");

  }

}

// transforms matrix of weights from 0.0 -> N to 0.0 -> 1.0 no matter how large the heaviest weight is.
fn normalize_weights(weights: &Vec<Vec<fp>>) -> Vec<Vec<fp>> {
  let mut heaviest_weight: fp = 0.0 as fp;
  for row in weights.iter() {
    for num in row.iter() {
      if *num > heaviest_weight {
        heaviest_weight = *num;
      }
    }
  }

  let corrective_ratio = 1.0 as fp / heaviest_weight;

  let mut normalized_weights = vec![];
  for row in weights.iter() {
    let mut normalized_row = vec![];
    for num in row.iter() {
      normalized_row.push(
        *num * corrective_ratio
      );
    }
    normalized_weights.push( normalized_row );
  }
  normalized_weights
}

fn html_format_tour_details(weights: &Vec<Vec<fp>>, brute_tour: &Vec<CityNum>) -> String {
  let mut s = "<pre>".to_string();
  let n = weights.len();
  for row_i in 0..n {
    s += "    ";
    for col_i in 0..n {
        if row_i == col_i {
          s += "x.x         ";
          continue;
        }
        s += format!("{:0.8}  ", weights[row_i][col_i] ).as_str();
    }
    s += "<br/>";
  }
  s += format!("tour = {:?}", brute_tour).as_str();
  s += "</pre>";
  s
}


fn print_square_matrix(weights: &Vec<Vec<fp>>) {
  let n = weights.len();
  for row_i in 0..n {
    print!("    ");
    for col_i in 0..n {
        if row_i == col_i {
          print!("x.x         ");
          continue;
        }
        print!("{:0.8}  ", weights[row_i][col_i] );
    }
    println!("");
  }
}

