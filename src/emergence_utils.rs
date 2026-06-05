//! emergence_utils.rs
//! helper flat-index table that emergence.rs uses once field_dsl.rs is patched to re-export.
#[inline]
pub fn dsl_idx(x: u32, y: u32, z: u32, nx: u32, ny: u32, nz: u32) -> usize {
    let stride_x = 1u32;
    let stride_y = nx;
    let stride_z = nx * ny;
    (z * stride_z + y * stride_y + x * stride_x) as usize
}
