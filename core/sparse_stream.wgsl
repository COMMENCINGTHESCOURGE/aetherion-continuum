//   M : H : τ  —  morton, hash, temporal coherence
//   three axes. remove one, streaming degrades to dense.

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

struct IndirectDispatch {
    x: u32, y: u32, z: u32,
};

// ═══ VINCULUM BARS ═══

const MORTON_BITS: u32     = 21u;          // ¯axis|morton — bits per axis
const MORTON_MASK: u32     = 0x1FFFFFu;    // ¯21bit|overflow
const HASH_PRIME: u32      = 16777213u;    // ¯collision|uniform
const HASH_SCALE: f32      = 1048576.0;    // ¯world|grid
const COHERENCE_FLOOR: f32 = 0.3;          // ¯inactive|active
const COHERENCE_STEEPNESS: f32 = 2.0;      // ¯distance→weight curve
const CELL_RATIO: f32      = 8.0;          // ¯cellsize|prediction window
const WORKGROUP_TILE: u32  = 64u;          // ¯threads|dispatch

// ═══ BINDINGS ═══

@group(0) @binding(0) var<storage, read_write> nodes: array<SparseNode>;
@group(0) @binding(1) var<storage, read_write> hash_table: array<SpatialHashEntry>;
@group(0) @binding(2) var<uniform> stream_req: StreamRequest;
@group(0) @binding(3) var<storage, read_write> active_count: atomic<u32>;
@group(0) @binding(4) var<storage, read_write> indirect_dispatch: IndirectDispatch;

// ═══ MORTON ═══

fn morton_encode(pos: vec3<u32>) -> u64 {
    var x = u64(pos.x & MORTON_MASK);
    var y = u64(pos.y & MORTON_MASK);
    var z = u64(pos.z & MORTON_MASK);

    x = (x | (x << 32u)) & 0x1F00000000FFFFu;
    x = (x | (x << 16u)) & 0x1F0000FF0000FFu;
    x = (x | (x << 8u))  & 0x100F00F00F00F00Fu;
    x = (x | (x << 4u))  & 0x10C30C30C30C30C3u;
    x = (x | (x << 2u))  & 0x1249249249249249u;

    y = (y | (y << 32u)) & 0x1F00000000FFFFu;
    y = (y | (y << 16u)) & 0x1F0000FF0000FFu;
    y = (y | (y << 8u))  & 0x100F00F00F00F00Fu;
    y = (y | (y << 4u))  & 0x10C30C30C30C30C3u;
    y = (y | (y << 2u))  & 0x1249249249249249u;

    z = (z | (z << 32u)) & 0x1F00000000FFFFu;
    z = (z | (z << 16u)) & 0x1F0000FF0000FFu;
    z = (z | (z << 8u))  & 0x100F00F00F00F00Fu;
    z = (z | (z << 4u))  & 0x10C30C30C30C30C3u;
    z = (z | (z << 2u))  & 0x1249249249249249u;

    return x | (y << 1u) | (z << 2u);
}

// ═══ SPATIAL HASH ═══

fn hash_position(pos: vec3<f32>) -> u32 {
    let grid = vec3<u32>(
        u32(pos.x * HASH_SCALE) & MORTON_MASK,
        u32(pos.y * HASH_SCALE) & MORTON_MASK,
        u32(pos.z * HASH_SCALE) & MORTON_MASK,
    );
    let morton = morton_encode(grid);
    return u32(morton ^ (morton >> 32u)) % HASH_PRIME;
}

// ═══ COHERENCE PREDICTION ═══

fn predict_coherence(node: SparseNode, camera_pos: vec3<f32>, camera_vel: vec3<f32>, dt: f32) -> f32 {
    let decoded = vec3<f32>(
        f32(node.morton_code & u64(MORTON_MASK)),
        f32((node.morton_code >> u32(MORTON_BITS)) & u64(MORTON_MASK)),
        f32((node.morton_code >> u32(MORTON_BITS * 2u)) & u64(MORTON_MASK)),
    );
    let cell_size = 1.0 / f32(1u << node.depth);
    let center = decoded * cell_size;
    let predicted = center + camera_vel * dt;
    let dist_to_camera = length(predicted - camera_pos);
    let prediction_err = dist_to_camera / (cell_size * CELL_RATIO);

    return 1.0 / (1.0 + pow(prediction_err, COHERENCE_STEEPNESS));
}

// ═══ ACTIVATION ═══

@compute @workgroup_size(64)
fn sparse_stream_activate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let node_idx = gid.x;
    if node_idx >= arrayLength(&nodes) { return; }

    var node = nodes[node_idx];

    // stream_req.min_corner = camera position, max_corner = camera position + velocity * dt
    // So camera_vel = (max_corner - min_corner) / dt
    let dt = 0.016;
    let camera_vel = (stream_req.max_corner - stream_req.min_corner) / dt;
    let coherence = predict_coherence(
        node,
        stream_req.min_corner,
        camera_vel,
        dt,
    );
    node.temporal_coherence = coherence;

    if coherence > COHERENCE_FLOOR {
        let old_idx = atomicAdd(&active_count, 1u);
        let decoded = vec3<f32>(
            f32(node.morton_code & u64(MORTON_MASK)),
            f32((node.morton_code >> u32(MORTON_BITS)) & u64(MORTON_MASK)),
            f32((node.morton_code >> u32(MORTON_BITS * 2u)) & u64(MORTON_MASK)),
        );
        let h = hash_position(decoded * 0.000001);
        let entry = SpatialHashEntry(h, node_idx, hash_table[h].next);
        hash_table[h] = entry;
    }

    nodes[node_idx] = node;
}

// ═══ INDIRECT DISPATCH BUILD ═══

@compute @workgroup_size(64)
fn build_indirect_dispatch(@builtin(global_invocation_id) gid: vec3<u32>) {
    let node_idx = gid.x;
    if node_idx >= arrayLength(&nodes) { return; }

    let node = nodes[node_idx];
    if node.temporal_coherence <= COHERENCE_FLOOR { return; }

    // Each active node contributes WORKGROUP_TILE threads worth of work
    // Accumulate total workgroups atomically
    let workgroups_per_node = (WORKGROUP_TILE + WORKGROUP_TILE - 1u) / WORKGROUP_TILE; // = 1
    let old_count = atomicAdd(&indirect_dispatch.x, workgroups_per_node);
    
    // Only the last thread to add writes the final y,z (they're always 1,1)
    if old_count == 0u {
        indirect_dispatch.y = 1u;
        indirect_dispatch.z = 1u;
    }
}
