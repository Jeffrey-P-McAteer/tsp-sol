
use super::*;

pub fn solve(node_coordinates: &Vec<(usize, f32, f32)>, weights: &Vec<Vec<f32>>) -> Vec<usize> {
  // We begin with points 0, 1, and 2.
  // These will be overwritten in the largest-triangle-fining process
  let mut ordered_visits: Vec<usize> = vec![0, 1, 2]; // holds the path as a vector of indexes relating to the city number beginning at 0
  
  
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
  }
  
  { // Print solution
    //print_path_metadata(&ordered_visits, &weights);
    //save_state_image(format!("./views/{}.png", ordered_visits.len()), &ordered_visits, &node_coordinates, &center);
    //println!("ordered_visits = {:?}", ordered_visits);
    //println!("unordered_visits = {:?}", unordered_visits);
  }
  
  return ordered_visits;
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


