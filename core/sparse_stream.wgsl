// sparse_stream.wgsl — GOVERNOR: consumed by zero_sync_dispatch.rs in emit dispatch path.
struct Header { active: u32, capacity: u32, epoch: u32, flags: u32 };
@group(0) @binding(0) var<storage,read>    keys: array<u64>;
@group(0) @binding(1) var<storage,read_write> counts: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> header: Header;
@compute @workgroup_size(64)
fn histogram_pass(@builtin(global_invocation_id) gid: vec3u) {
  let i = gid.x;
  if (i >= header.capacity) { return; }
  let h = (keys[i] >> 32) ^ (keys[i] & 0xffffffffu);
  atomicAdd(&counts[h & 1023u], 1u);
}
