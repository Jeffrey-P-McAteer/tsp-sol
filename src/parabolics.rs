
// used to solve the 6-parameter equation of parabolics given some points
// (A*(x^2)) + (B*x*y) + (C*(y^2)) + (D*x) + (E*y) + F = 0

use super::*;

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

    let f = Parabola { a: 1.0, b: 1.0, c: 1.0, d: 1.0, e: 1.0, f: 1.0 };
    let dom = Domain::with_dim(f.dim().value());
    let mut solver = TrustRegion::new(&f, &dom);

    // Initial guess.
    let mut x = na::vector![y1]; // wait wat?

    let mut fx = na::vector![0.0];

    for i in 1..=100 {
        let res = solver.next(&f, &dom, &mut x, &mut fx);

        if let Err(e) = res {
            println!("Error! e={e:?}");
            break;
        }

        println!(
            "iter = {}\t|| fx || = {}\tx = {:?}",
            i,
            fx.norm(),
            x.as_slice()
        );

        if fx.norm() < 1e-5 {
            println!("solved");
            //return Ok(());
            break;
        }
    }

    // Tons todo around here

    return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
}

// Given an X value, return ALL y values for the coefficients. Possible results are
// and empty Vec, 1 value, or 2 values. I guess for all 0s you could have infinite values as well,
// but this fn will represent that as a vec of 3 values.
pub fn evaluate_parabolic_for_x(x: fp, (a, b, c, d, e, f): (fp, fp, fp, fp, fp, fp)) -> Vec<fp> {
    let mut y_vals: Vec<fp> = vec![];

    return y_vals;
}
