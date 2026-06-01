//   ρ : φ : ψ : ∇T : ∇M : C
//   six terms. delete one, the field collapses.

struct FieldCell {
    rho: f32,
    phi: f32,
    psi: f32,
    grad_T: vec3<f32>,
    grad_M: vec3<f32>,
    C: f32,
};

struct ConservationInvariants {
    mass_total: atomic<f32>,
    energy_total: atomic<f32>,
    momentum_x: atomic<f32>,
    momentum_y: atomic<f32>,
    momentum_z: atomic<f32>,
    divergence_max: atomic<f32>,
};

struct DispatchMeta {
    tile_count: u32,
    cells_per_tile: u32,
    active_mask: u32,
    thermal_limit_pct: f32,
    vram_pressure_pct: f32,
};

// ═══ VINCULUM BARS ═══
// each pair is a measurement: delete one, the bar breaks.

const PHASE_SF_BAR: f32 = 1.0;        // ¯solid|fluid
const PHASE_FG_BAR: f32 = 1.0;        // ¯fluid|gas
const PHASE_SOLID: f32 = 0.0;
const PHASE_FLUID: f32 = PHASE_SOLID + PHASE_SF_BAR;
const PHASE_GAS: f32   = PHASE_FLUID + PHASE_FG_BAR;

const COHESION_FLOOR: f32   = 0.15;   // ¯degenerate|stable
const COHESION_HARDEN: f32  = 0.0001; // ¯C→C' per step
const HARDEN_START: f32     = 0.01;   // ¯inert|active

const PSI_COUPLING_BAR: f32     = 0.25;   // ¯self|neighbor
const PSI_THRESHOLD: f32        = 0.001;  // ¯decoupled|coupled
const PSI_DIVERGENCE_BAR: f32   = 0.5;    // ¯flux|entanglement

const MOISTURE_FLUX_MIN: f32    = 0.001;  // ¯static|flowing
const MOISTURE_DECAY_BAR: f32   = 0.998;  // ¯∇M→∇M' per step

const FG_OVERHEAT_BAR: f32 = 2.0;        // ¯boil|superheat

const DT: f32 = 0.016;

// ═══ BINDINGS ═══

@group(0) @binding(0) var<storage, read_write> field: array<FieldCell>;
@group(0) @binding(1) var<storage, read_write> invariants: ConservationInvariants;
@group(0) @binding(2) var<uniform> meta: DispatchMeta;
@group(0) @binding(3) var<storage, read> phase_diagram: array<vec4<f32>>;

// ═══ PHASE TRANSITION ═══

fn phase_transition(cell: ptr<function, FieldCell>, dt: f32) {
    let c = (*cell).C;
    let phi = (*cell).phi;
    let rho = (*cell).rho;
    let temp_mag = length((*cell).grad_T);

    let band = u32(c * 255.0);
    let thresholds = phase_diagram[band];
    let solid_fluid_T = thresholds.x;
    let fluid_gas_T = thresholds.y;
    let latent_sf = thresholds.z;
    let latent_fg = thresholds.w;

    let cohesion_factor: f32 = select(1.0, c / COHESION_FLOOR, c < COHESION_FLOOR);

    var new_phi = phi;
    if temp_mag > solid_fluid_T && phi < PHASE_FLUID {
        let blend = min((temp_mag - solid_fluid_T) / (fluid_gas_T - solid_fluid_T), 1.0);
        new_phi = mix(phi, PHASE_FLUID, blend * cohesion_factor);
        (*cell).rho -= latent_sf * blend * cohesion_factor * dt;
    }
    if temp_mag > fluid_gas_T && phi < PHASE_GAS {
        let blend = min((temp_mag - fluid_gas_T) / (fluid_gas_T * FG_OVERHEAT_BAR), 1.0);
        new_phi = mix(new_phi, PHASE_GAS, blend * cohesion_factor);
        (*cell).rho -= latent_fg * blend * cohesion_factor * dt;
    }

    (*cell).phi = new_phi;
}

// ═══ DIVERGENCE ═══

fn compute_divergence(cell: FieldCell, neighbors: array<FieldCell, 6>) -> f32 {
    var div: f32 = 0.0;
    div += neighbors[0].rho * neighbors[0].grad_T.x - neighbors[1].rho * neighbors[1].grad_T.x;
    div += neighbors[2].rho * neighbors[2].grad_T.y - neighbors[3].rho * neighbors[3].grad_T.y;
    div += neighbors[4].rho * neighbors[4].grad_T.z - neighbors[5].rho * neighbors[5].grad_T.z;
    div *= 1.0 + cell.psi * PSI_DIVERGENCE_BAR;
    return abs(div);
}

// ═══ MAIN KERNEL ═══

@compute @workgroup_size(8, 8, 1)
fn field_tensor_update(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell_idx = gid.x + gid.y * 64u + gid.z * 4096u;
    if cell_idx >= meta.cell_count() { return; }

    var cell = field[cell_idx];

    phase_transition(&cell, DT);

    let psi_coupling = cell.psi * PSI_COUPLING_BAR;
    if psi_coupling > PSI_THRESHOLD {
        let nx = (gid.x + 1u) % 64u;
        let px = (gid.x + 63u) % 64u;
        let ny = (gid.y + 1u) % 64u;
        let py = (gid.y + 63u) % 64u;

        let n_idx = nx + gid.y * 64u + gid.z * 4096u;
        let p_idx = px + gid.y * 64u + gid.z * 4096u;
        let n_y_idx = gid.x + ny * 64u + gid.z * 4096u;
        let p_y_idx = gid.x + py * 64u + gid.z * 4096u;

        let neighbor_rho = (field[n_idx].rho + field[p_idx].rho + field[n_y_idx].rho + field[p_y_idx].rho) * PSI_COUPLING_BAR;
        cell.rho = mix(cell.rho, neighbor_rho, psi_coupling);
        cell.phi = mix(cell.phi, (field[n_idx].phi + field[p_idx].phi) * 0.5, psi_coupling);
    }

    if cell.C > HARDEN_START {
        cell.C = min(cell.C + cell.C * COHESION_HARDEN, 1.0);
    }

    let moisture_flux = length(cell.grad_M);
    if moisture_flux > MOISTURE_FLUX_MIN {
        cell.grad_M *= MOISTURE_DECAY_BAR;
    }

    field[cell_idx] = cell;

    atomicAdd(&invariants.mass_total, cell.rho);
    atomicAdd(&invariants.energy_total, cell.phi * cell.rho);
}

fn meta_cell_count(meta: DispatchMeta) -> u32 {
    return meta.tile_count * meta.cells_per_tile;
}
