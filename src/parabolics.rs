
// used to solve the 6-parameter equation of parabolics given some points
// (A*(x^2)) + (B*x*y) + (C*(y^2)) + (D*x) + (E*y) + F = 0

use super::*;

/*
// See https://github.com/datamole-ai/gomez/blob/main/examples/rosenbrock.rs
use nalgebra as na;
//use gomez::nalgebra as na;
use gomez::prelude::*;
use gomez::solver::TrustRegion;
use na::{Dim, DimName, IsContiguous};

struct Parabola {
    a: fp,
    b: fp,
    c: fp,
    d: fp,
    e: fp,
    f: fp,
}

impl Problem for Parabola {
    type Scalar = fp;
    type Dim = na::U1; // Because we get out a single scalar given X in... wait?

    fn dim(&self) -> Self::Dim {
        na::U1::name()
    }
}

impl System for Parabola {
    fn eval<Sx, Sfx>(
        &self,
        x: &na::Vector<Self::Scalar, Self::Dim, Sx>,
        fx: &mut na::Vector<Self::Scalar, Self::Dim, Sfx>,
    ) -> Result<(), ProblemError>
    where
        Sx: na::storage::Storage<Self::Scalar, Self::Dim> + IsContiguous,
        Sfx: na::storage::StorageMut<Self::Scalar, Self::Dim>,
    {
        // Cheated w/ wolframalpha eq solver,
        // solve (A*(x^2)) + (B*x*y) + (C*(y^2)) + (D*x) + (E*y) + F = 0 for y
        let x = x[0];
        let (a,b,c,d,e,f) = (self.a, self.b, self.c, self.d, self.e, self.f);
        if c != 0.0 {
            // We have a + and - y value (same magnitude, just signs flipped)
            fx[0] = -(
                    ( ((b*x) + e).powf(2.0) - (4.0*c*( (x*((a*x) + d)) + f)) ).sqrt() + (b*x) + e
                ) / (
                    2.0*c
            );

            // TODO flip sign on TOP .sqrt() expression and store... ???
            
            Ok(())
        }
        else if c == 0.0 && (b*x) + e != 0.0 {
            // ??? no idea what this means
            fx[0] = -( (x*( (self.a*x) + d )) + self.f ) / ( (b*x) + e );
            Ok(())
        }
        else {
            // We're a circle / out of the domain!
            Err(ProblemError::InvalidValue) // todo maybe good Custom() type w/ domain msg
        }
    }
}

*/


/*
// OLD solve_for_6pts gaussian

    // Adapted from https://github.com/nerdypepper/gauss_jordan/blob/master/src/main.rs
    // Solve Ax^2 + Bxy + Cy^2 + Dx + Ey + F == 0
    const SIZE: usize = 6;
    let mut system: [[fp; SIZE+1]; SIZE]  = [
        [ x1.powf(2.0), x1*y1, y1.powf(2.0), x1, y1, 1.0, 0.0],
        [ x2.powf(2.0), x2*y2, y2.powf(2.0), x2, y2, 1.0, 0.0],
        [ x3.powf(2.0), x3*y3, y3.powf(2.0), x3, y3, 1.0, 0.0],
        [ x4.powf(2.0), x4*y4, y4.powf(2.0), x4, y4, 1.0, 0.0],
        [ x5.powf(2.0), x5*y5, y5.powf(2.0), x5, y5, 1.0, 0.0],
        [ x6.powf(2.0), x6*y6, y6.powf(2.0), x6, y6, 1.0, 0.0]
    ];

    for i in 0..SIZE-1 {
        for j in i..SIZE-1 {
            if system[i][i] == 0.0 {
                continue;
            }
            else {
                let factor = system[j + 1][i] as fp / system[i][i] as fp;
                for k in i..SIZE+1 {
                    system[j + 1][k] -= factor * system[i][k] as fp;
                }
            }
        }
    }

    // System is now in row-echelon form

    for i in (1..SIZE).rev() {
        if system[i][i] == 0.0 {
            continue;
        }
        else {
            for j in (1..i+1).rev() {
                let factor = system[j - 1][i] as fp / system[i][i] as fp;
                for k in (0..SIZE+1).rev() {
                    system[j - 1][k] -= factor * system[i][k] as fp;
                }
            }
        }
    }

    for i in 0..SIZE {
        if system[i][i] == 0.0 {
            continue; // println!("Infnitely many solutions");
        }
        else {
            system[i][SIZE] /= system[i][i] as fp;
            system[i][i] = 1.0;
            //println!("X{} = {}", i + 1, system[i][SIZE]);
        }
    }

    // System is now solved, the values in system[0..6][SIZE] are the value of A,B,C,D,E,F

    println!("");
    println!("system = {:?}", system);
    println!("");
    for row in system {
        println!(" >  {:?}", row);
    }
    println!("");
    
    //return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    return (
        system[0][SIZE], system[1][SIZE], system[2][SIZE],
        system[3][SIZE], system[4][SIZE], system[5][SIZE]
    );

*/

// Borrowed from https://stackoverflow.com/questions/73356686/rust-for-loop-with-floating-point-types
fn float_loop(start: fp, threshold: fp, step_size: fp) -> impl Iterator<Item = fp> {
    std::iter::successors(Some(start), move |&prev| {
        let next = prev + step_size;
        (next < threshold).then_some(next)
    })
}



pub fn solve_for_6pts(
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
    const RANGE_BEGIN: fp = -5.0;
    const RANGE_END: fp = 5.0;
    //const RANGE_STEP: fp = 0.1;
    const RANGE_STEP: fp = 1.0;

    let mut best_abcdef = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let mut smallest_error = 99999999.0;

    for a in float_loop(RANGE_BEGIN, RANGE_END, RANGE_STEP) {
        for b in float_loop(RANGE_BEGIN, RANGE_END, RANGE_STEP) {
            for c in float_loop(RANGE_BEGIN, RANGE_END, RANGE_STEP) {
                for d in float_loop(RANGE_BEGIN, RANGE_END, RANGE_STEP) {
                    for e in float_loop(RANGE_BEGIN, RANGE_END, RANGE_STEP) {
                        for f in float_loop(RANGE_BEGIN, RANGE_END, RANGE_STEP) {
                            // On top of the world, baby
                            
                            let c_y1 = evaluate_parabolic_for_x_absonly(x1, (a,b,c,d,e,f));
                            let c_y2 = evaluate_parabolic_for_x_absonly(x2, (a,b,c,d,e,f));
                            let c_y3 = evaluate_parabolic_for_x_absonly(x3, (a,b,c,d,e,f));
                            let c_y4 = evaluate_parabolic_for_x_absonly(x4, (a,b,c,d,e,f));
                            let c_y5 = evaluate_parabolic_for_x_absonly(x5, (a,b,c,d,e,f));
                            let c_y6 = evaluate_parabolic_for_x_absonly(x6, (a,b,c,d,e,f));
                            
                            let this_error = (c_y1 - y1).abs() +
                                             (c_y2 - y2).abs() +
                                             (c_y3 - y3).abs() +
                                             (c_y4 - y4).abs() +
                                             (c_y5 - y5).abs() +
                                             (c_y6 - y6).abs();
                            
                            if this_error < smallest_error {
                                best_abcdef = (a, b, c, d, e, f); // store lowest error!
                                smallest_error = this_error;
                            }

                        }
                    }
                }
            }
        }
    }

    return best_abcdef;
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



