//   ρ : φ : ψ : C    and    ∇T : ∇M
//   two vec4<f32> per cell, split across field + gradient buffers.
//   all shaders share this layout — no struct size mismatches.

struct ConservationState {
    mass_drift: f32,
    energy_drift: f32,
    momentum_drift: vec3<f32>,
    total_mass: f32,
    total_energy: f32,
};

struct DispatchMeta {
    tile_count: u32,
    cells_per_tile: u32,
    active_mask: u32,
    thermal_limit_pct: f32,
    vram_pressure_pct: f32,
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

const CELLS_PER_TILE: u32 = 4096u;

// ═══ BINDINGS ═══

@group(0) @binding(0) var<storage, read_write> field: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> state: ConservationState;
@group(0) @binding(2) var<uniform> meta: DispatchMeta;
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
    div += neighbors[0].x * neighbor_grads[0].x - neighbors[1].x * neighbor_grads[1].x;
    div += neighbors[2].x * neighbor_grads[2].y - neighbors[3].x * neighbor_grads[3].y;
    div += neighbors[4].x * neighbor_grads[4].z - neighbors[5].x * neighbor_grads[5].z;
    div *= 1.0 + cell.z * PSI_DIVERGENCE_BAR;
    return abs(div);
}

// ═══ MAIN KERNEL ═══

@compute @workgroup_size(8, 8, 1)
fn field_tensor_update(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell_idx = gid.x + gid.y * 64u + gid.z * 4096u;
    let total_cells = meta.tile_count * meta.cells_per_tile;
    if cell_idx >= total_cells { return; }

    var cell = field[cell_idx];
    var grad = gradient[cell_idx];

    phase_transition(&cell, &grad, DT);

    let psi = cell.z;
    let psi_coupling = psi * PSI_COUPLING_BAR;
    if psi_coupling > PSI_THRESHOLD {
        let nx = (gid.x + 1u) % 64u;
        let px = (gid.x + 63u) % 64u;
        let ny = (gid.y + 1u) % 64u;
        let py = (gid.y + 63u) % 64u;

        let n_idx = nx + gid.y * 64u + gid.z * 4096u;
        let p_idx = px + gid.y * 64u + gid.z * 4096u;
        let n_y_idx = gid.x + ny * 64u + gid.z * 4096u;
        let p_y_idx = gid.x + py * 64u + gid.z * 4096u;

        let nbr_rho_n = field[n_idx].x;
        let nbr_rho_p = field[p_idx].x;
        let nbr_rho_ny = field[n_y_idx].x;
        let nbr_rho_py = field[p_y_idx].x;
        let nbr_phi_n = field[n_idx].y;
        let nbr_phi_p = field[p_idx].y;

        let neighbor_rho = (nbr_rho_n + nbr_rho_p + nbr_rho_ny + nbr_rho_py) * PSI_COUPLING_BAR;
        cell.x = mix(cell.x, neighbor_rho, psi_coupling);
        cell.y = mix(cell.y, (nbr_phi_n + nbr_phi_p) * 0.5, psi_coupling);
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
}
