// sparse_stream.wgsl — Sparse octree activation and indirect dispatch builder
// Bindings match bgl_sparse in zero_sync_dispatch.rs: 
//   @0 storage read_write sparse_nodes
//   @1 storage read_write spatial_hash
//   @2 uniform stream_req
//   @3 storage read_write active_count (atomic)
//   @4 storage read_write indirect_dispatch

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
    _pad: f32,     // alignment padding
    _pad2: vec2<f32>,
};

struct IndirectDispatch {
    x: u32,
    y: u32,
    z: u32,
};

@group(0) @binding(0) var<storage, read_write> nodes: array<SparseNode>;
@group(0) @binding(1) var<storage, read_write> hash_table: array<SpatialHashEntry>;
@group(0) @binding(2) var<uniform> stream_req: StreamRequest;
@group(0) @binding(3) var<storage, read_write> active_count: atomic<u32>;
@group(0) @binding(4) var<storage, read_write> indirect_dispatch: IndirectDispatch;

const MAX_NODES: u32 = 65536u;

// Morton code helpers
fn expand_bits(v: u32) -> u32 {
    var x = v;
    x = (x | (x << 16u)) & 0x030000FFu;
    x = (x | (x << 8u))  & 0x0300F00Fu;
    x = (x | (x << 4u))  & 0x030C30C3u;
    x = (x | (x << 2u))  & 0x09249249u;
    return x;
}

fn morton3(x: u32, y: u32, z: u32) -> u32 {
    return expand_bits(x) | (expand_bits(y) << 1u) | (expand_bits(z) << 2u);
}

fn hash_position(pos: vec3<f32>, cell_size: f32) -> u32 {
    let ix = u32(pos.x / cell_size);
    let iy = u32(pos.y / cell_size);
    let iz = u32(pos.z / cell_size);
    return morton3(ix, iy, iz);
}

@compute @workgroup_size(64)
fn sparse_stream_activate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    
    // Per-cell activation: mark nodes within stream request bounds
    // Each thread maps to a potential node in the spatial hash
    let cell_size = stream_req.min_detail * 2.0;
    let grid_dim = u32(1.0 / cell_size) + 1u;
    
    if (idx >= grid_dim * grid_dim * grid_dim) { return; }
    
    let z = idx / (grid_dim * grid_dim);
    let y = (idx / grid_dim) % grid_dim;
    let x = idx % grid_dim;
    
    let pos = vec3<f32>(
        f32(x) * cell_size,
        f32(y) * cell_size,
        f32(z) * cell_size
    );
    
    // Check if within stream request bounds
    if (pos.x < stream_req.min_corner.x || pos.x > stream_req.max_corner.x ||
        pos.y < stream_req.min_corner.y || pos.y > stream_req.max_corner.y ||
        pos.z < stream_req.min_corner.z || pos.z > stream_req.max_corner.z) {
        return;
    }
    
    let hash = hash_position(pos, cell_size);
    let slot = hash % 1024u;
    
    // Linear probe spatial hash
    for (var probe = 0u; probe < 16u; probe++) {
        let entry_idx = (slot + probe) % 1024u;
        if (hash_table[entry_idx].hash == 0u) {
            // Claim this entry
            hash_table[entry_idx].hash = hash;
            hash_table[entry_idx].node_idx = idx;
            hash_table[entry_idx].next = 0u;
            
            // Initialize sparse node
            nodes[idx].morton_code = u64(hash);
            nodes[idx].child_mask = 0u;
            nodes[idx].field_offset = idx * 64u; // offset into field buffer
            nodes[idx].parent_idx = 0u;
            nodes[idx].depth = 0u;
            nodes[idx].temporal_coherence = 1.0;
            nodes[idx].padding = vec2(0u);
            
            atomicAdd(&active_count, 1u);
            break;
        }
    }
}

@compute @workgroup_size(1)
fn build_indirect_dispatch(@builtin(global_invocation_id) gid: vec3<u32>) {
    let active = atomicLoad(&active_count);
    let groups = (active + 63u) / 64u;
    
    indirect_dispatch = IndirectDispatch(
        max(1u, groups),
        1u,
        1u
    );
}
