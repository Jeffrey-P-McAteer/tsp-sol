
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
    // We brute force everything, taking the smallest error (abs()'ing all Y values) for the 6 points
    
    /*const coef_range: &[fp] = &[
        -2.0, -1.9, -1.8, -1.7, -1.6, -1.5, -1.4, -1.3, -1.2, -1.1,
        -1.0, -0.9, -0.8, -0.7, -0.6, -0.5, -0.4, -0.3, -0.2, -0.1,
        0.0,
        0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
        2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.0,
    ];*/

    const coef_range: &[fp] = &[
        -10.0, -9.0, -8.0, -7.0, -6.0, -5.0, -4.0, -3.0, -2.0, -1.0,
               -9.5, -8.5, -7.5, -6.5, -5.5, -4.5, -3.5, -2.5, -1.5,
        0.0,
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
        1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5,
    ];

    const big_coef_range: &[fp] = &[
        -12.0, -10.0, -8.0, -6.0, -4.0, -2.0,
        -1.0, 0.0, 1.0,
        2.0, 4.0, 6.0, 8.0, 10.0, 12.0
    ];

    let mut best_abcdef = Arc::new(Mutex::new( (0.0, 0.0, 0.0, 0.0, 0.0, 0.0) ));
    let mut smallest_error = Arc::new(Mutex::new( 99999999.0 ));

    for a in coef_range {
        // Copy vars to be moved into thread
        let (x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6) = (x1, y1, x2, y2, x3, y3, x4, y4, x5, y5, x6, y6);
        let best_abcdef = best_abcdef.clone();
        let smallest_error = smallest_error.clone();
        thread_pool.execute(move || {
            
            let mut local_best_abcdef = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
            let mut local_smallest_error = 99999999.0;

            for b in coef_range {
                for c in coef_range {
                    for d in coef_range {
                        for e in coef_range {
                            for f in big_coef_range { // F can move more b/c I want angular accuracy
                                
                                let this_coefs = (*a,*b,*c,*d,*e,*f);
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

                            }
                        }
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



