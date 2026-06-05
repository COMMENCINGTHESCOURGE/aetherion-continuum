// pillar_2_volumetric_simulation.wgsl — GOVERNOR: loaded by zero_sync_dispatch.rs
struct Bindings {
  field: array<vec4<f32>>,
  out: array<vec4<f32>>,
  header: vec4u,
}

@group(0) @binding(0) var<storage,read_write> b: Bindings;

@compute @workgroup_size(64)
fn pillar_2(@builtin(global_invocation_id) gid: vec3u) {
  // volumetric_simulation: deterministic 3x3->1x3 reduction on tile basis 2
  var acc = vec3f(0.0);
  for (var j = 0u; j < 9u; j = j + 1u) {
    acc = acc + b.field[(gid.x * 9u + j) % arrayLength(&b.field)].xyz;
  }
  b.out[gid.x] = vec4f(acc, 1.0);
}
