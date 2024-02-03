
// used to solve the 6-parameter equation of parabolics given some points
// (A*(x^2)) + (B*x*y) + (C*(y^2)) + (D*x) + (E*y) + F = 0

/* https://www.varsitytutors.com/hotmath/hotmath_help/topics/conic-sections-and-standard-forms-of-equations

 The general equation for any conic section is

 Ax2+Bxy+Cy2+Dx+Ey+F=0

   where A,B,C,D,E and F are constants.

As we change the values of some of the constants, the shape of the corresponding conic will also change.  It is important to know the differences in the equations to help quickly identify the type of conic that is represented by a given equation.
      If B2−4AC is less than zero, if a conic exists, it will be either a circle or an ellipse.
      If B2−4AC equals zero, if a conic exists, it will be a parabola.
      If B2−4AC is greater than zero, if a conic exists, it will be a hyperbola.

*/



#![allow(non_upper_case_globals)]
#![allow(unused_mut)]

use super::*;

use std::sync::{Mutex, RwLock, Arc}; // 48-core-xeon threading go brrrr

const NUM_THREADS: usize = 32;
const GPU_THREAD_BLOCKS: usize = 1024;

pub fn solve_for_6pts(
  thread_pool: &ThreadPool,
  gpu_device: &mut Option<wgpu::Adapter>,
  (x1, y1): (fp, fp),
  (x2, y2): (fp, fp),
  (x3, y3): (fp, fp),
  (x4, y4): (fp, fp),
  (x5, y5): (fp, fp),
  (x6, y6): (fp, fp),
)
    -> (fp, fp, fp, fp, fp, fp)
{

    const min_guess: fp = -2000.0;
    const max_guess: fp = 2000.0;
    let guess_range = max_guess - min_guess;

    let mut best_abcdef = Arc::new(Mutex::new( (0.0, 0.0, 0.0, 0.0, 0.0, 0.0) ));
    let mut smallest_error = Arc::new(Mutex::new( 99999999.0 ));

    // const error_exit_target: fp = 0.30; // randomly permute until we hit < this error
    // const long_iter_error_exit_target: fp = 0.95;
    // const long_iter_count: usize = 5_000_000_000;

    const error_exit_target: fp = 0.11; // randomly permute until we hit < this error
    const long_iter_error_exit_target: fp = 0.18;
    const long_iter_count: usize = 9_000_000_000;

    if let Some(ref mut gpu_device) = gpu_device {

    }
    else {
        // Fall back to CPU
        println!("Falling back to CPU in solve_for_6pts!");
        for _ in 0..NUM_THREADS {
            // Copy vars to be moved into thread
            let (x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6) = (x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6);
            let best_abcdef = best_abcdef.clone();
            let smallest_error = smallest_error.clone();
            thread_pool.execute(move || {

                let mut local_best_abcdef = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
                let mut local_smallest_error = 99999999.0;

                let mut loop_i = 0;
                loop {
                    loop_i += 1;

                    let a = (fastrand::f32() * guess_range) + min_guess;
                    let b = (fastrand::f32() * guess_range) + min_guess;
                    let c = (fastrand::f32() * guess_range) + min_guess;
                    let d = (fastrand::f32() * guess_range) + min_guess;
                    let e = (fastrand::f32() * guess_range) + min_guess;
                    let f = (fastrand::f32() * guess_range) + min_guess;

                    let shp_test = (b*b) - (4.0*a*c);

                    let is_hyperbola = shp_test > 0.0;
                    if !is_hyperbola {
                        continue; // We only want hyperbola formulas!
                    }

                    // let is_parabola = shp_test < 0.001 && shp_test > -0.001; // test near-zero b/c of how our brute force is setup; TODO math the others out of the equation!
                    // if !is_parabola {
                    //     continue; // We only want parabola formulas!
                    // }

                    let this_coefs = (a,b,c,d,e,f);
                    let c_y1 = evaluate_parabolic_for_x_absonly(x1, this_coefs);
                    let c_y2 = evaluate_parabolic_for_x_absonly(x2, this_coefs);
                    let c_y3 = evaluate_parabolic_for_x_absonly(x3, this_coefs);
                    let c_y4 = evaluate_parabolic_for_x_absonly(x4, this_coefs);
                    let c_y5 = evaluate_parabolic_for_x_absonly(x5, this_coefs);
                    let c_y6 = evaluate_parabolic_for_x_absonly(x6, this_coefs);

                    let this_error = (c_y1 - y1.abs()).abs() +
                                     (c_y2 - y2.abs()).abs() +
                                     (c_y3 - y3.abs()).abs() +
                                     (c_y4 - y4.abs()).abs() +
                                     (c_y5 - y5.abs()).abs() +
                                     (c_y6 - y6.abs()).abs();

                    if this_error < local_smallest_error {
                        local_best_abcdef = this_coefs;
                        local_smallest_error = this_error;
                    }

                    if local_smallest_error < error_exit_target {
                        break; // we're done, other threads will check in 5,000 or so random checks and exit.
                    }

                    if loop_i % 2_000_000 == 0 {
                        // Should we exit b/c another thread found & exited?
                        let mut smallest_err_guard = smallest_error.lock().unwrap();
                        if *smallest_err_guard < error_exit_target {
                            break;
                        }
                        if loop_i > long_iter_count {
                            if *smallest_err_guard < long_iter_error_exit_target {
                                break;
                            }
                        }
                    }

                    if loop_i > long_iter_count {
                        if this_error < long_iter_error_exit_target {
                            local_best_abcdef = this_coefs;
                            local_smallest_error = this_error;
                        }
                        if local_smallest_error < long_iter_error_exit_target {
                            break; // we're done, other threads will check in 5,000 or so random checks and exit.
                        }
                    }

                }

                {
                    let mut smallest_err_guard = smallest_error.lock().unwrap();
                    let mut best_abcdef_guard = best_abcdef.lock().unwrap();
                    if local_smallest_error < *smallest_err_guard {
                        *smallest_err_guard = local_smallest_error;
                        *best_abcdef_guard = local_best_abcdef;
                    }
                }
                // Mutexes are unlocked

            });
        }

        thread_pool.join();
    }

    println!("Curve Error: {}", *smallest_error.lock().unwrap() );

    return *(best_abcdef.lock().unwrap());

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

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct GPUData {
    xy_array: [fp; 12],
    best_abcdef: [fp; 6],
    smallest_error: fp,
}

