@group(0)
@binding(0)
var<storage, read_write> v_indices: array<u32, 1000000>; // Assuming 1 million elements


@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let width: u32 = 1000u;  // If the width of your data structure is 1000
    let linear_index = global_id.y * width + global_id.x;

    // Ensure we don't access out of bounds
    if (linear_index < 1000000u) {

        v_indices[linear_index] = v_indices[linear_index] + 1u;

    }

}
