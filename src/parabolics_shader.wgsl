
// WGPU only supports up to vec4<T>; there are no nice ways to pass arbitrary-length
// arrays!
// ergo, the mess below.
/*struct SingleGpuThreadData {
  xy_array_pt1: vec4<f32>, // xy_array: [fp; 12],
  xy_array_pt2: vec4<f32>,
  xy_array_pt3: vec4<f32>,

  best_abcdef_pt1: vec4<f32>, // best_abcdef: [fp; 6],
  best_abcdef_pt2: vec2<f32>,

  smallest_error: f32, // smallest_error: fp,
};*/

const BYTES_PER_T = (6*2) + 6 + 1;

@group(0)
@binding(0)
var<storage, read_write> gpu_data: array<f32>; // this is used as both input and output for convenience

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // TODO write search in terms of gpu_data[global_id.x] data.

    gpu_data[(global_id.x * ((6*2) + 6 + 1) ) + ((6*2) + 6) ] = 123.456;  // Point at last f32 in per-thread array

}
