//   ρ : φ : ψ : C    and    ∇T : ∇M
//   two vec4<f32> per cell, split across field + gradient buffers.
//   all shaders share this layout — no struct size mismatches.

struct ConservationState {
    mass_drift_fixed: atomic<i32>,
    total_mass_fixed: atomic<u32>,
    energy_drift_fixed: atomic<i32>,
    total_energy_fixed: atomic<u32>,
    momentum_drift_x: atomic<i32>,
    momentum_drift_y: atomic<i32>,
    momentum_drift_z: atomic<i32>,
    saturation_count: atomic<u32>, // increments when clamping occurs
};

struct DispatchMeta {
    tile_count: u32,
    cells_per_tile: u32,
    active_mask: u32,
    thermal_limit_pct: f32,
    vram_pressure_pct: f32,
    grid_size_x: u32,
    grid_size_y: u32,
    grid_size_z: u32,
};

// ═══ VINCULUM BARS ═══

const PHASE_SF_BAR: f32 = 1.0;
const PHASE_FG_BAR: f32 = 1.0;
const PHASE_SOLID: f32 = 0.0;
const PHASE_FLUID: f32 = PHASE_SOLID + PHASE_SF_BAR;
const PHASE_GAS: f32   = PHASE_FLUID + PHASE_FG_BAR;

const COHESION_FLOOR: f32   = 0.15;
const COHESION_HARDEN: f32  = 0.0001;
const HARDEN_START: f32     = 0.01;

const PSI_COUPLING_BAR: f32     = 0.25;
const PSI_THRESHOLD: f32        = 0.001;
const PSI_DIVERGENCE_BAR: f32   = 0.5;

const MOISTURE_FLUX_MIN: f32    = 0.001;
const MOISTURE_DECAY_BAR: f32   = 0.998;

const FG_OVERHEAT_BAR: f32 = 2.0;

const DT: f32 = 0.016;

const MASS_DRIFT_SCALE: f32 = 1000.0; // fixed-point scale for atomic accumulation

// ═══ BINDINGS ═══

@group(0) @binding(0) var<storage, read_write> field: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> state: ConservationState;
@group(0) @binding(2) var<uniform> meta_: DispatchMeta;
@group(0) @binding(3) var<storage, read> phase_diagram: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> gradient: array<vec4<f32>>;

// ═══ PHASE TRANSITION ═══

fn phase_transition(cell: ptr<function, vec4<f32>>, grad: ptr<function, vec4<f32>>, dt: f32) {
    let rho = (*cell).x;
    let phi = (*cell).y;
    let C = (*cell).w;
    let temp_mag = length((*grad).xyz);

    let band = u32(C * 255.0);
    let thresholds = phase_diagram[band];
    let solid_fluid_T = thresholds.x;
    let fluid_gas_T = thresholds.y;
    let latent_sf = thresholds.z;
    let latent_fg = thresholds.w;

    let cohesion_factor: f32 = select(1.0, C / COHESION_FLOOR, C < COHESION_FLOOR);

    var new_phi = phi;
    if temp_mag > solid_fluid_T && phi < PHASE_FLUID {
        let blend = min((temp_mag - solid_fluid_T) / (fluid_gas_T - solid_fluid_T), 1.0);
        new_phi = mix(phi, PHASE_FLUID, blend * cohesion_factor);
        (*cell).x -= latent_sf * blend * cohesion_factor * dt;
    }
    if temp_mag > fluid_gas_T && phi < PHASE_GAS {
        let blend = min((temp_mag - fluid_gas_T) / (fluid_gas_T * FG_OVERHEAT_BAR), 1.0);
        new_phi = mix(new_phi, PHASE_GAS, blend * cohesion_factor);
        (*cell).x -= latent_fg * blend * cohesion_factor * dt;
    }

    (*cell).y = new_phi;
}

// ═══ DIVERGENCE ═══

fn compute_divergence(cell: vec4<f32>, grad: vec4<f32>, neighbors: array<vec4<f32>, 6>, neighbor_grads: array<vec4<f32>, 6>) -> f32 {
    var div: f32 = 0.0;
    // neighbors order: +x, -x, +y, -y, +z, -z
    div += neighbors[0].x * neighbor_grads[0].x - neighbors[1].x * neighbor_grads[1].x;
    div += neighbors[2].x * neighbor_grads[2].y - neighbors[3].x * neighbor_grads[3].y;
    div += neighbors[4].x * neighbor_grads[4].z - neighbors[5].x * neighbor_grads[5].z;
    div *= 1.0 + cell.z * PSI_DIVERGENCE_BAR;
    return div; // signed divergence (not abs) — accumulate sign-aware
}

// Helper: atomic add with saturation for atomic<i32>
fn atomicAddSaturating(ptr: ptr<storage, atomic<i32>>, delta: i32) {
    // clamp to safe bounds to avoid overflow; use 31-bit range to be conservative
    let MIN_I: i32 = -2147483647;
    let MAX_I: i32 = 2147483647;
    loop {
        let old = atomicLoad(ptr);
        let sum = old + delta;
        // if sum exceeds bounds, clamp and increment saturation counter
        var clamped = sum;
        if (sum > MAX_I) {
            clamped = MAX_I;
            atomicAdd(&state.saturation_count, 1u);
        } else if (sum < MIN_I) {
            clamped = MIN_I;
            atomicAdd(&state.saturation_count, 1u);
        }
        let res = atomicCompareExchangeWeak(ptr, old, clamped);
        if (res.exchanged) { break; }
        // otherwise, loop and retry (handle spurious failures)
    }
}

// ═══ MAIN KERNEL ═══

@compute @workgroup_size(8, 8, 1)
fn field_tensor_update(@builtin(global_invocation_id) gid: vec3<u32>) {
    let sizeX = meta_.grid_size_x;
    let sizeY = meta_.grid_size_y;
    let sizeZ = meta_.grid_size_z;

    let plane = sizeX * sizeY;
    let cell_idx = gid.x + gid.y * sizeX + gid.z * plane;
    let total_cells = meta_.tile_count * meta_.cells_per_tile;
    if cell_idx >= total_cells { return; }

    var cell = field[cell_idx];
    var grad = gradient[cell_idx];

    phase_transition(&cell, &grad, DT);

    let psi = cell.z;
    let psi_coupling = psi * PSI_COUPLING_BAR;
    if psi_coupling > PSI_THRESHOLD {
        // compute neighbor indices with wrap-around
        let nx = (gid.x + 1u) % sizeX;
        let px = (gid.x + (sizeX - 1u)) % sizeX;
        let ny = (gid.y + 1u) % sizeY;
        let py = (gid.y + (sizeY - 1u)) % sizeY;
        let nz = (gid.z + 1u) % sizeZ;
        let pz = (gid.z + (sizeZ - 1u)) % sizeZ;

        let idx_px = px + gid.y * sizeX + gid.z * plane;
        let idx_nx = nx + gid.y * sizeX + gid.z * plane;
        let idx_py = gid.x + py * sizeX + gid.z * plane;
        let idx_ny = gid.x + ny * sizeX + gid.z * plane;
        let idx_pz = gid.x + gid.y * sizeX + pz * plane;
        let idx_nz = gid.x + gid.y * sizeX + nz * plane;

        // Preload neighbor values BEFORE any write to prevent load-after-store hazard
        let nbr_rho_px = field[idx_px].x;
        let nbr_rho_nx = field[idx_nx].x;
        let nbr_rho_py = field[idx_py].x;
        let nbr_rho_ny = field[idx_ny].x;
        let nbr_rho_pz = field[idx_pz].x;
        let nbr_rho_nz = field[idx_nz].x;

        let nbr_phi_px = field[idx_px].y;
        let nbr_phi_nx = field[idx_nx].y;
        let nbr_phi_py = field[idx_py].y;
        let nbr_phi_ny = field[idx_ny].y;
        let nbr_phi_pz = field[idx_pz].y;
        let nbr_phi_nz = field[idx_nz].y;

        let neighbor_rho = (nbr_rho_nx + nbr_rho_px + nbr_rho_ny + nbr_rho_py + nbr_rho_nz + nbr_rho_pz) * (PSI_COUPLING_BAR / 6.0);
        cell.x = mix(cell.x, neighbor_rho, psi_coupling);

        let neighbor_phi = (nbr_phi_nx + nbr_phi_px + nbr_phi_ny + nbr_phi_py + nbr_phi_nz + nbr_phi_pz) / 6.0;
        cell.y = mix(cell.y, neighbor_phi, psi_coupling);
    }

    if cell.w > HARDEN_START {
        cell.w = min(cell.w + cell.w * COHESION_HARDEN, 1.0);
    }

    let moisture_flux = grad.w;
    if moisture_flux > MOISTURE_FLUX_MIN {
        grad.w *= MOISTURE_DECAY_BAR;
    }

    field[cell_idx] = cell;
    gradient[cell_idx] = grad;

    // Accumulate signed divergence into state buffer using fixed-point atomic<i32>
    // gather neighbor grad/rho for divergence computation
    var neighbors: array<vec4<f32>, 6>;
    var neighbor_grads: array<vec4<f32>, 6>;

    // +x, -x
    let nx2 = (gid.x + 1u) % sizeX;
    let px2 = (gid.x + (sizeX - 1u)) % sizeX;
    let idx_xp = nx2 + gid.y * sizeX + gid.z * plane;
    let idx_xm = px2 + gid.y * sizeX + gid.z * plane;
    neighbors[0] = field[idx_xp];
    neighbors[1] = field[idx_xm];
    neighbor_grads[0] = gradient[idx_xp];
    neighbor_grads[1] = gradient[idx_xm];

    // +y, -y
    let ny2 = (gid.y + 1u) % sizeY;
    let py2 = (gid.y + (sizeY - 1u)) % sizeY;
    let idx_yp = gid.x + ny2 * sizeX + gid.z * plane;
    let idx_ym = gid.x + py2 * sizeX + gid.z * plane;
    neighbors[2] = field[idx_yp];
    neighbors[3] = field[idx_ym];
    neighbor_grads[2] = gradient[idx_yp];
    neighbor_grads[3] = gradient[idx_ym];

    // +z, -z
    let nz2 = (gid.z + 1u) % sizeZ;
    let pz2 = (gid.z + (sizeZ - 1u)) % sizeZ;
    let idx_zp = gid.x + gid.y * sizeX + nz2 * plane;
    let idx_zm = gid.x + gid.y * sizeX + pz2 * plane;
    neighbors[4] = field[idx_zp];
    neighbors[5] = field[idx_zm];
    neighbor_grads[4] = gradient[idx_zp];
    neighbor_grads[5] = gradient[idx_zm];

    let div = compute_divergence(cell, grad, neighbors, neighbor_grads);

    // convert to fixed-point and atomically accumulate (signed)
    let driftFixed: i32 = i32(round(div * MASS_DRIFT_SCALE));
    atomicAddSaturating(&state.mass_drift_fixed, driftFixed);
}
