//   active : budget : dispatch
//   three counters. delete one, indirect dispatch degrades to brute force.
//   This shader reads the indirect_dispatch built by sparse_stream_activate/build_indirect_dispatch
//   and validates/copies it to the dispatch_cmd buffer for the field tensor passes.

<<<<<<< HEAD
struct DispatchMeta {
    total_cells: u32,
    active_cells: u32,
    budget: u32,
    frame: u32,
};

struct SparseNode {
    morton_code: u64,
    child_mask: u32,
    field_offset: u32,
    parent_idx: u32,
    depth: u32,
    temporal_coherence: f32,
    active: u32,
    priority: f32,
};

struct DispatchCmd {
    x: u32,
    y: u32,
    z: u32,
=======
struct IndirectDispatch {
    x: u32, y: u32, z: u32,
>>>>>>> 04b758c59f52fde8c978f3958d883ed06cde6006
};

const MAX_NODES: u32 = 1024u;

@group(0) @binding(0) var<storage, read_write> nodes: array<SparseNode>;
@group(0) @binding(1) var<storage, read_write> hash_table: array<SpatialHashEntry>;
@group(0) @binding(2) var<uniform> stream_req: StreamRequest;
@group(0) @binding(3) var<storage, read_write> active_count: atomic<u32>;
@group(0) @binding(4) var<storage, read_write> indirect_dispatch: IndirectDispatch;

// Re-declare SparseNode and SpatialHashEntry from sparse_stream for layout compatibility
struct SparseNode {
    morton_code: u64,
    child_mask: u32,
    field_offset: u32,
    parent_idx: u32,
    depth: u32,
    temporal_coherence: f32,
    padding: vec2<u32>,
};

struct SpatialHashEntry {
    hash: u32,
    node_idx: u32,
    next: u32,
};

struct StreamRequest {
    min_corner: vec3<f32>,
    max_corner: vec3<f32>,
    min_detail: f32,
    temporal_budget_ms: f32,
};

@compute @workgroup_size(1)
fn build_indirect(@builtin(global_invocation_id) gid: vec3<u32>) {
    // The indirect_dispatch is already built by sparse_stream's build_indirect_dispatch
    // This pass just ensures it's valid (non-zero) for the field tensor passes
    if indirect_dispatch.x == 0u {
        indirect_dispatch = IndirectDispatch(1u, 1u, 1u);
    }
}
