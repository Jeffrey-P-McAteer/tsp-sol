
// used to solve the 6-parameter equation of parabolics given some points
// (A*(x^2)) + (B*x*y) + (C*(y^2)) + (D*x) + (E*y) + F = 0

use super::*;

use std::sync::{Mutex, RwLock, Arc}; // 48-core-xeon threading go brrrr

pub fn solve_for_6pts(
  thread_pool: &ThreadPool,
  (x1, y1): (fp, fp),
  (x2, y2): (fp, fp),
  (x3, y3): (fp, fp),
  (x4, y4): (fp, fp),
  (x5, y5): (fp, fp),
  (x6, y6): (fp, fp),
)
    -> (fp, fp, fp, fp, fp, fp)
{
  use linreg::{linear_regression, linear_regression_of};

  // See https://docs.rs/linreg/latest/linreg/

  // for a,b,c,d,e,f approximate their values and use linear_regression_of
  // N times feeding in the x,y pairs above to linearly best-fit their true values
  // using a prior set of best-fit linear coeficients
  const NUM_ATTEMPTS: usize = 100;
  let mut r1 = (1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
  let mut r2 = (1.0, 1.0, 1.0, 1.0, 1.0, 1.0);

  for a in NUM_ATTEMPTS {
    if a % 2 == 0 {
        // x and y values stored as tuples
      let tuples: Vec<(f32, f32)> = vec![(1.0, 2.0),
                                         (2.0, 4.0),
                                         (3.0, 5.0),
                                         (4.0, 4.0),
                                         (5.0, 5.0)];
      
      if let Ok(r) = linear_regression_of(&tuples) {
        
      }
    }
    else {

    }
  }

  // return average of best coeficients
  return (
    (r1.0 + r2.0) / 2.0,
    (r1.1 + r2.1) / 2.0,
    (r1.2 + r2.2) / 2.0,
    (r1.3 + r2.3) / 2.0,
    (r1.4 + r2.4) / 2.0,
    (r1.5 + r2.5) / 2.0,
  );
}

// Given an X value, return ALL y values for the coefficients. Possible results are
// and empty Vec, 1 value, or 2 values. I guess for all 0s you could have infinite values as well,
// but this fn will represent that as a vec of 3 values.
pub fn evaluate_parabolic_for_x(x: fp, (a, b, c, d, e, f): (fp, fp, fp, fp, fp, fp)) -> Vec<fp> {
    let mut y_vals: Vec<fp> = vec![];

    // WA: solve (A*(x^2)) + (B*x*y) + (C*(y^2)) + (D*x) + (E*y) + F = 0 for y

    if c != 0.0 {
        y_vals.push(
            -(
                ( ((b*x) + e).powf(2.0) - (4.0*c*( (x*((a*x) + d)) + f)) ).sqrt() + (b*x) + e
                ) / (
                    2.0*c
            )
        );
        y_vals.push(
            -(
                -(( ((b*x) + e).powf(2.0) - (4.0*c*( (x*((a*x) + d)) + f)) ).sqrt()) + (b*x) + e
                ) / (
                    2.0*c
            )
        );
    }

    if c == 0.0 && ((b*x)+e) != 0.0 {
        y_vals.push(
            -( (x*( (a*x) + d )) + f ) / ( (b*x) + e )
        );
    }


    return y_vals;
}


// Faster, incorrect version of evaluate_parabolic_for_x
#[inline]
pub fn evaluate_parabolic_for_x_absonly(x: fp, (a, b, c, d, e, f): (fp, fp, fp, fp, fp, fp)) -> fp {
    let mut y = 0.0;

    if c != 0.0 {
        y = (-(
                ( ((b*x) + e).powf(2.0) - (4.0*c*( (x*((a*x) + d)) + f)) ).sqrt() + (b*x) + e
                ) / (
                    2.0*c
            )).abs();
    }

    if c == 0.0 && ((b*x)+e) != 0.0 {
        y = (
            -( (x*( (a*x) + d )) + f ) / ( (b*x) + e )
        ).abs();
    }

    return y;
}



