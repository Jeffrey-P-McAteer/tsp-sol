
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
    
    const coef_range: &[fp] = &[
        // python -c "print(', '.join([ f'{x*0.25}' for x in range(-40, 40) ]))"
        -10.0, -9.75, -9.5, -9.25, -9.0, -8.75, -8.5, -8.25, -8.0, -7.75, -7.5, -7.25, -7.0, -6.75, -6.5, -6.25, -6.0, -5.75, -5.5, -5.25, -5.0, -4.75, -4.5, -4.25, -4.0, -3.75, -3.5, -3.25, -3.0, -2.75, -2.5, -2.25, -2.0, -1.75, -1.5, -1.25, -1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 2.75, 3.0, 3.25, 3.5, 3.75, 4.0, 4.25, 4.5, 4.75, 5.0, 5.25, 5.5, 5.75, 6.0, 6.25, 6.5, 6.75, 7.0, 7.25, 7.5, 7.75, 8.0, 8.25, 8.5, 8.75, 9.0, 9.25, 9.5, 9.75

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
                            for f in coef_range {
                                
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



