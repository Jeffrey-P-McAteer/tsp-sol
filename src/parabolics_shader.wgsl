
// const FLOATS_PER_GPU_STEP = (6*2) + 6 + 1;

@group(0)
@binding(0)
var<storage, read_write> gpu_data: array<f32>; // this is used as both input and output for convenience

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // TODO write search in terms of gpu_data[global_id.x] data.

    gpu_data[(global_id.x * FLOATS_PER_GPU_STEP ) + FLOATS_PER_GPU_STEP - 1 ] = 123.456;  // Point at last f32 in per-thread array

}
