
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
    use linregress::{FormulaRegressionBuilder, RegressionDataBuilder};

    let y = vec![y1, y2, y3, y4, y5, y6];
    let v_x1 = vec![x1, x2, x3, x4, x5, x6];
    let v_x2 = vec![729.53, 439.0367, 42.054, 1., 0.];
    let v_x3 = vec![258.589, 616.297, 215.061, 498.361, 0.];
    
    let data = vec![("Y", y), ("X1", v_x1), ("X2", v_x2), ("X3", v_x3), ("X3", v_x3), ("X3", v_x3), ("X3", v_x3)];
    
    let data = RegressionDataBuilder::new().build_from(data).expect("Could not RegressionDataBuilder::new().build_from(data)");
    
    let formula = "Y ~ X1 + X2 + X3 + X4 + X5 + X6";
    
    let model = FormulaRegressionBuilder::new()
        .data(&data)
        .formula(formula)
        .fit().expect("Could not .fit() data");
    
    let parameters: Vec<_> = model.iter_parameter_pairs().collect(); // coefficients... I think
    let pvalues: Vec<_> = model.iter_p_value_pairs().collect();
    let standard_errors: Vec<_> = model.iter_se_pairs().collect();

    return (
        parameters[0].1,
        parameters[1].1,
        parameters[2].1,
        parameters[3].1,
        parameters[4].1,
        parameters[5].1,
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



