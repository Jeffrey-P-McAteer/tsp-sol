
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
//const GPU_THREAD_BLOCKS: usize = 1024;
const GPU_THREAD_BLOCKS: usize = 16;
const FLOATS_PER_GPU_STEP: usize = (6*2) + 6 + 1;

const PARABOLICS_SHADER_CODE: &'static str = include_str!("parabolics_shader.wgsl");

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

    const error_exit_target: fp = 0.16; // randomly permute until we hit < this error
    const long_iter_error_exit_target: fp = 0.86;
    const long_iter_count: usize = 5_000_000;

    const squeese_amount: fp = 4.0;

    const max_search_seconds: u64 = 120;

    let solve_begin_t = std::time::SystemTime::now();

    if let Some(ref mut gpu_device) = gpu_device {
        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults()
        };
        if let Ok((ref mut device, ref mut queue)) = futures::executor::block_on(gpu_device.request_device(&device_desc, None)) {

            let mut replaced_shader_source_code = PARABOLICS_SHADER_CODE.to_string();
            replaced_shader_source_code = replaced_shader_source_code.replace("FLOATS_PER_GPU_STEP", format!("{}", FLOATS_PER_GPU_STEP).as_str() );

            let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                //source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(PARABOLICS_SHADER_CODE)),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from( replaced_shader_source_code )),
            });

            //let gpu_data = AllGpuThreadData::default();

            // 12 xy fps, 6 abcdef fps, and a min size fp.
            //let gpu_data: [fp; GPU_THREAD_BLOCKS * FLOATS_PER_GPU_STEP ] = [0.0; GPU_THREAD_BLOCKS * ((6*2) + 6 + 1) ];

            //let gpu_data: Box<[fp; GPU_THREAD_BLOCKS * ((6*2) + 6 + 1) ]> = Box::new([0.0; GPU_THREAD_BLOCKS * ((6*2) + 6 + 1) ]);
            let gpu_data: [fp; GPU_THREAD_BLOCKS * FLOATS_PER_GPU_STEP ] = [0.0; GPU_THREAD_BLOCKS * FLOATS_PER_GPU_STEP ];
            let gpu_data = Box::<[fp; GPU_THREAD_BLOCKS * FLOATS_PER_GPU_STEP]>::pin(gpu_data); // might make vulkan happier to not have data move?

            //let size = std::mem::size_of_val(&gpu_data) as wgpu::BufferAddress;
            let size = std::mem::size_of::<[fp; GPU_THREAD_BLOCKS * FLOATS_PER_GPU_STEP]>() as wgpu::BufferAddress;

            println!("gpu_data = {:?} size = {:?}\n^^ BEGIN ^^", gpu_data, size);


            // Instantiates buffer without data.
            // `usage` of buffer specifies how it can be used:
            //   `BufferUsage::MAP_READ` allows it to be read (outside the shader).
            //   `BufferUsage::COPY_DST` allows it to be the destination of the copy.
            let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Instantiates buffer with data (`gpu_data`).
            // Usage allowing the buffer to be:
            //   A storage buffer (can be bound within a bind group and thus available to a shader).
            //   The destination of a copy.
            //   The source of a copy.
            let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("StorageBuffer"),
                contents: bytemuck::cast_slice(&gpu_data[..]),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });

            // A bind group defines how buffers are accessed by shaders.
            // It is to WebGPU what a descriptor set is to Vulkan.
            // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

            // A pipeline specifies the operation of a shader

            // Instantiates the pipeline.
            let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &cs_module,
                entry_point: "main",
            });

            // Instantiates the bind group, once again specifying the binding of buffers.
            let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: storage_buffer.as_entire_binding(),
                }],
            });

            // A command encoder executes one or many pipelines.
            // It is to WebGPU what a command buffer is to Vulkan.
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
                cpass.set_pipeline(&compute_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.insert_debug_marker("compute nearest polynominal fit");
                cpass.dispatch_workgroups(gpu_data.len() as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
            }
            // Sets adds copy operation to command encoder.
            // Will copy data from storage buffer on GPU to staging buffer on CPU.
            encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);

            device.poll(wgpu::Maintain::Wait); // Let data settle into place

            // Submits command encoder for processing
            let work_to_send = Some(encoder.finish());
            device.poll(wgpu::Maintain::Wait); // Let data settle into place
            let sub_idx = queue.submit(work_to_send);

            device.poll(wgpu::Maintain::WaitForSubmissionIndex(sub_idx)); // Blocks until sub_idx's work has completed

            // Note that we're not calling `.await` here.
            let buffer_slice = staging_buffer.slice(..);
            // Gets the future representing when `staging_buffer` can be read from
            buffer_slice.map_async(wgpu::MapMode::Read, move |_| { } );

            // Poll the device in a blocking manner so that our future resolves.
            // In an actual application, `device.poll(...)` should
            // be called in an event loop or on another thread.
            device.poll(wgpu::Maintain::Wait);

            // Awaits until `buffer_future` can be read from
            //{
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result: Vec<[fp; FLOATS_PER_GPU_STEP]> = data
                .chunks_exact( std::mem::size_of::<[fp; FLOATS_PER_GPU_STEP ]>() ) // size of one GPU threads block of numbers
                .map(|b| {
                    unsafe {
                        std::mem::transmute::<[u8; std::mem::size_of::<[fp; FLOATS_PER_GPU_STEP]>() ], [fp; FLOATS_PER_GPU_STEP]>(
                            b.try_into().expect("data.chunks_exact fucked up the calc for std::mem::size_of::<[fp; (6*2) + 6 + 1]>() ")
                        ).clone()
                    }
                } )
                .collect();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            staging_buffer.unmap(); // Unmaps buffer from memory
                                    // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                    //   delete myPointer;
                                    //   myPointer = NULL;
                                    // It effectively frees the memory

            // Returns data from buffer
            // result

            // TODO use result!
            println!("DONE! result = {:?}\n^^ END ^^", result);

            // }

            device.poll(wgpu::Maintain::Wait);
            device.destroy();
            println!("After device.destroy()!");

            /*else {
                println!("failed to run compute on gpu!")
            }*/


        }
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

                let mut guess_divisor: (fp, fp, fp, fp, fp, fp) = (1.0, 1.0, 1.0, 1.0, 1.0, 1.0);

                let mut loop_i = 0;
                loop {

                    loop_i += 1;

                    let a = (fastrand::f32() * (guess_range / guess_divisor.0) ) + (min_guess / guess_divisor.0);
                    let b = (fastrand::f32() * (guess_range / guess_divisor.1) ) + (min_guess / guess_divisor.1);
                    let c = (fastrand::f32() * (guess_range / guess_divisor.2) ) + (min_guess / guess_divisor.2);
                    let d = (fastrand::f32() * (guess_range / guess_divisor.3) ) + (min_guess / guess_divisor.3);
                    let e = (fastrand::f32() * (guess_range / guess_divisor.4) ) + (min_guess / guess_divisor.4);
                    let f = (fastrand::f32() * (guess_range / guess_divisor.5) ) + (min_guess / guess_divisor.5);

                    // let mut l_guess_range = guess_range / (f32::max(1.0, loop_i as f32 / 100.0));
                    // if l_guess_range < 2.0 {
                    //     loop_i = 0; // re-start scanning from high fuzziness -> low fuzziness
                    // }
                    // // Force guess range to be huge while error is huge
                    // if local_smallest_error > 9.0 && l_guess_range < 50.0 {
                    //     l_guess_range = guess_range / (f32::max(1.0, loop_i as f32 / 1000.0));
                    // }

                    // Min guess is just average of all weights - 0.5 l_guess_range
                    // let l_min_guess = ((local_best_abcdef.0+
                    //                     local_best_abcdef.1+
                    //                     local_best_abcdef.2+
                    //                     local_best_abcdef.3+
                    //                     local_best_abcdef.4+
                    //                     local_best_abcdef.5) / 6.0) - (l_guess_range / 2.0);
                    // The above calculations ensure the amount of random deltas get smaller and smaller,
                    // and instead of looking up best(<a million wide guesses>) we get
                    // a set of best(<a hundred wide guesses>) -> best(<a permutation of the previous best w/ smaller guess deltas>) -> ....

                    // We mutate all variables if local_smallest_error is > 4.0 (or if at a beginning iteration), and then
                    // We now mutate only 1 variable at a time from the best-case scenario
                    // let a = local_best_abcdef.0 + if local_smallest_error > 9.0 || loop_i < 10000 || loop_i % 6 == 0 { ((fastrand::f32() * l_guess_range) - (local_best_abcdef.0 - (l_guess_range / 2.0)) ) } else { 0.0 };
                    // let b = local_best_abcdef.1 + if local_smallest_error > 9.0 || loop_i < 10000 || loop_i % 6 == 1 { ((fastrand::f32() * l_guess_range) - (local_best_abcdef.1 - (l_guess_range / 2.0)) ) } else { 0.0 };
                    // let c = local_best_abcdef.2 + if local_smallest_error > 9.0 || loop_i < 10000 || loop_i % 6 == 2 { ((fastrand::f32() * l_guess_range) - (local_best_abcdef.2 - (l_guess_range / 2.0)) ) } else { 0.0 };
                    // let d = local_best_abcdef.3 + if local_smallest_error > 9.0 || loop_i < 10000 || loop_i % 6 == 3 { ((fastrand::f32() * l_guess_range) - (local_best_abcdef.3 - (l_guess_range / 2.0)) ) } else { 0.0 };
                    // let e = local_best_abcdef.4 + if local_smallest_error > 9.0 || loop_i < 10000 || loop_i % 6 == 4 { ((fastrand::f32() * l_guess_range) - (local_best_abcdef.4 - (l_guess_range / 2.0)) ) } else { 0.0 };
                    // let f = local_best_abcdef.5 + if local_smallest_error > 9.0 || loop_i < 10000 || loop_i % 6 == 5 { ((fastrand::f32() * l_guess_range) - (local_best_abcdef.5 - (l_guess_range / 2.0)) ) } else { 0.0 };

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
                        // tighten the most-modified 3 variables this iteration
                        let avg_modification_amnt: fp = (
                            (a - local_best_abcdef.0).abs() +
                            (b - local_best_abcdef.1).abs() +
                            (c - local_best_abcdef.2).abs() +
                            (d - local_best_abcdef.3).abs() +
                            (e - local_best_abcdef.4).abs() +
                            (f - local_best_abcdef.5).abs()
                        ) / 6.0;
                        if (a - local_best_abcdef.0).abs() >= avg_modification_amnt {
                            guess_divisor = (
                                guess_divisor.0 * (2.0 * squeese_amount),
                                guess_divisor.1,
                                guess_divisor.2,
                                guess_divisor.3,
                                guess_divisor.4,
                                guess_divisor.5
                            );
                        }
                        if (b - local_best_abcdef.1).abs() >= avg_modification_amnt {
                            guess_divisor = (
                                guess_divisor.0,
                                guess_divisor.1 * (2.0 * squeese_amount),
                                guess_divisor.2,
                                guess_divisor.3,
                                guess_divisor.4,
                                guess_divisor.5
                            );
                        }
                        if (c - local_best_abcdef.2).abs() >= avg_modification_amnt {
                            guess_divisor = (
                                guess_divisor.0,
                                guess_divisor.1,
                                guess_divisor.2 * (2.0 * squeese_amount),
                                guess_divisor.3,
                                guess_divisor.4,
                                guess_divisor.5
                            );
                        }
                        if (d - local_best_abcdef.3).abs() >= avg_modification_amnt {
                            guess_divisor = (
                                guess_divisor.0,
                                guess_divisor.1,
                                guess_divisor.2,
                                guess_divisor.3 * (2.0 * squeese_amount),
                                guess_divisor.4,
                                guess_divisor.5
                            );
                        }
                        if (e - local_best_abcdef.4).abs() >= avg_modification_amnt {
                            guess_divisor = (
                                guess_divisor.0,
                                guess_divisor.1,
                                guess_divisor.2,
                                guess_divisor.3,
                                guess_divisor.4 * (2.0 * squeese_amount),
                                guess_divisor.5
                            );
                        }
                        if (f - local_best_abcdef.5).abs() >= avg_modification_amnt {
                            guess_divisor = (
                                guess_divisor.0,
                                guess_divisor.1,
                                guess_divisor.2,
                                guess_divisor.3,
                                guess_divisor.4,
                                guess_divisor.5 * (2.0 * squeese_amount)
                            );
                        }
                        local_best_abcdef = this_coefs;
                        local_smallest_error = this_error;
                        println!("a thread on loop_i={} guess_divisor={},{},{},{},{},{} lowered error to {}", loop_i,
                            guess_divisor.0, guess_divisor.1, guess_divisor.2, guess_divisor.3, guess_divisor.4, guess_divisor.5,
                            this_error
                        );
                    }

                    if local_smallest_error < error_exit_target {
                        break; // we're done, other threads will check in 5,000 or so random checks and exit.
                    }

                    if loop_i % (long_iter_count / 2) == 0 {
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

                    if loop_i > (long_iter_count*2) {
                        // Reduce range of random guesses and go back to initial state
                        loop_i = 0;
                        guess_divisor = (
                            guess_divisor.0 * squeese_amount,
                            guess_divisor.1 * squeese_amount,
                            guess_divisor.2 * squeese_amount,
                            guess_divisor.3 * squeese_amount,
                            guess_divisor.4 * squeese_amount,
                            guess_divisor.5 * squeese_amount
                        );
                        if (guess_range / ((guess_divisor.0+guess_divisor.1+guess_divisor.2+guess_divisor.3+guess_divisor.4+guess_divisor.5) / 6.0) ) < 0.02 {
                            if local_smallest_error < long_iter_error_exit_target {
                                break; // ran out of guessing space but we hit our goal!
                            }
                            else {
                                // Ran out of guessing space, did not hit goal, reset search parameters if under max run duration
                                if let Ok(solve_duration) = solve_begin_t.elapsed() {
                                    if solve_duration.as_secs() > max_search_seconds {
                                        println!("! Ran out of guessing space, did not hit goal, ran out of time!");
                                        println!("local_smallest_error = {}, want better than {}", local_smallest_error, long_iter_error_exit_target);
                                        break;
                                    }
                                }
                                println!("Ran out of guessing space, did not hit goal, reset search parameters");
                                println!("local_smallest_error = {}, want better than {}", local_smallest_error, long_iter_error_exit_target);
                                guess_divisor = (1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
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

/*
#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::NoUninit)]
struct SingleGpuThreadData {
    xy_array: [fp; 12],
    best_abcdef: [fp; 6],
    smallest_error: fp,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct AllGpuThreadData {
    thread_data: [SingleGpuThreadData; GPU_THREAD_BLOCKS],
}

unsafe impl bytemuck::NoUninit for AllGpuThreadData { } // Solemnly swear I am up to no good


impl Default for AllGpuThreadData {
    fn default() -> AllGpuThreadData {
        AllGpuThreadData {
            thread_data: [SingleGpuThreadData::default(); GPU_THREAD_BLOCKS]
        }
    }
}

impl SingleGpuThreadData {
    pub fn from_bytes(bytes: &[u8]) -> SingleGpuThreadData {
        // TODO safety engineering
        unsafe {
            std::mem::transmute::<[u8; std::mem::size_of::<SingleGpuThreadData>() ], SingleGpuThreadData>(
                bytes.try_into().expect("Did not get right chunk size of bytes for SingleGpuThreadData::from_bytes!")
            ).clone() // clone so we no longer ref the GPU bytes
        }
    }
}
*/


