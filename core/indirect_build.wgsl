//   active : budget : dispatch
//   three counters. delete one, indirect dispatch degrades to brute force.

struct SparseNode {
    cell_offset: u32,
    lod: u32,
    active: u32,
    priority: f32,
};

struct DispatchCmd {
    x: u32,
    y: u32,
    z: u32,
};

@group(0) @binding(0) var<storage, read> field: array<f32>;
@group(0) @binding(1) var<storage, read_write> sparse_nodes: array<SparseNode>;

@group(0) @binding(2) var<storage, read_write> dispatch_cmd: array<DispatchCmd>;
@group(0) @binding(3) var<uniform> meta: DispatchMeta;

const MAX_NODES: u32 = 1024u;
const BUDGET: u32 = 1500u;  // max bricks per frame (from hyperpoly meta-dispatcher)

struct DispatchMeta {
    total_cells: u32,
    active_cells: u32,
    budget: u32,
    frame: u32,
};

@compute @workgroup_size(256, 1, 1)
fn build_indirect(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= MAX_NODES { return; }

    let node = sparse_nodes[idx];
    if node.active == 0u { return; }

    // Each active node dispatches 8x8x1 workgroups per tile
    let groups_x = (meta.total_cells + 63u) / 64u;  // ceil(cells / 64)
    let groups_y = 1u;
    let groups_z = 1u;

    // Only the first active thread writes the dispatch — others accumulate
    if idx == 0u {
        var cmd = dispatch_cmd[0u];
        cmd.x = groups_x;
        cmd.y = groups_y;
        cmd.z = groups_z;
        dispatch_cmd[0u] = cmd;
    }
}
