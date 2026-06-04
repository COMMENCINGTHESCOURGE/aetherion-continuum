//   M : E : P  —  mass, energy, momentum
//   three invariants. violate one, the proof breaks.

struct ConservationState {
    mass_drift: f32,
    energy_drift: f32,
    momentum_drift: vec3<f32>,
    total_mass: f32,
    total_energy: f32,
}

struct CorrectionLog {
    cell_idx: u32,
    pre_mass: f32,
    post_mass: f32,
    divergence_detected: f32,
    timestamp: f32,
}

const MASS_EPSILON: f32     = 1e-5;
const ENERGY_EPSILON: f32   = 1e-4;
const MOMENTUM_EPSILON: f32 = 1e-4;
const MAX_CORRECTION_BAR: f32 = 0.01;

const ENERGY_CAP: f32       = 1e6;
const ENERGY_DAMP: f32      = 0.999;
const GLOBAL_SCALE_BAR: f32 = 0.001;
const DRIFT_MULTIPLIER: f32 = 100.0;

const COHESION_SOLID_FLOOR: f32 = 0.15;

// ═══ BINDINGS ═══

@group(0) @binding(0) var<storage, read_write> field: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> gradient: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> state: ConservationState;
@group(0) @binding(3) var<storage, read_write> correction_log: array<CorrectionLog>;
@group(0) @binding(4) var<storage, read_write> log_count: atomic<u32>;
@group(0) @binding(5) var<uniform> cell_count: u32;

// ═══ VINCULUM FUNCTIONS ═══

fn mod9Classify(ratio: f32) -> u32 {
    if !isFinite(ratio) { return 0u; }
    let mod = u32(abs(round(ratio * 100.0))) % 9u;
    return mod;
}

fn vinculumRatioClassify(ratio: f32) -> u32 {
    let mod = mod9Classify(ratio);
    if mod == 1u || mod == 4u || mod == 7u { return 1u; }  // STABLE
    if mod == 0u || mod == 3u || mod == 6u { return 2u; }  // BREACH
    return 3u;  // NEUTRAL
}

// ═══ ENFORCE ═══

@compute @workgroup_size(64)
fn enforce_conservation(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= cell_count { return; }

    var cell = field[idx];
    var grad = gradient[idx];

    let rho = cell.x;
    let C = cell.w;
    var correction_needed = false;

    // Compute local divergence using 6-connected neighbors (clamped at boundaries)
    let cell_count_f = f32(cell_count);
    let grid_dim = u32(pow(cell_count_f, 1.0/3.0)) + 1u;
    let z = idx / (grid_dim * grid_dim);
    let y = (idx % (grid_dim * grid_dim)) / grid_dim;
    let x = idx % grid_dim;

    var div: f32 = 0.0;
    // X neighbors
    if x > 0u {
        let px = idx - 1u;
        div += field[px].x * gradient[px].x;
    }
    if x < grid_dim - 1u {
        let nx = idx + 1u;
        div -= field[nx].x * gradient[nx].x;
    }
    // Y neighbors
    if y > 0u {
        let py = idx - grid_dim;
        div += field[py].x * gradient[py].y;
    }
    if y < grid_dim - 1u {
        let ny = idx + grid_dim;
        div -= field[ny].x * gradient[ny].y;
    }
    // Z neighbors
    if z > 0u {
        let pz = idx - grid_dim * grid_dim;
        div += field[pz].x * gradient[pz].z;
    }
    if z < grid_dim - 1u {
        let nz = idx + grid_dim * grid_dim;
        div -= field[nz].x * gradient[nz].z;
    }
    div = abs(div);

    if rho < 0.0 {
        cell.x = 0.0;
        correction_needed = true;
    }

    if cell.y < 0.01 && C < COHESION_SOLID_FLOOR {
        cell.w = COHESION_SOLID_FLOOR;
        correction_needed = true;
    }

    if cell.x * cell.y > ENERGY_CAP {
        cell.x *= ENERGY_DAMP;
        correction_needed = true;
    }

    if correction_needed {
        let log_idx = atomicAdd(&log_count, 1u);
        if log_idx < arrayLength(&correction_log) {
            correction_log[log_idx] = CorrectionLog(
                idx,
                rho,
                cell.x,
                div,
                0.0,
            );
        }
    }

    // Vinculum classification of local mass drift
    let mass_drift_local = abs(rho - 1.0 / cell_count_f);
    let mass_classification = vinculumRatioClassify(mass_drift_local * 1000.0);

    // Accumulate mass for global correction pass
    atomicAdd(&state.total_mass, cell.x);

    field[idx] = cell;
    gradient[idx] = grad;
}

// ═══ GLOBAL ═══

@compute @workgroup_size(1)
fn global_correction_pass_vinculum() {
    let mass_drift = state.total_mass - 1.0;
    if abs(mass_drift) > MASS_EPSILON * DRIFT_MULTIPLIER {
        state.mass_drift = mass_drift;
    }

    let mass_ratio = mass_drift / max(state.total_mass, 0.001);
    let mass_classification = vinculumRatioClassify(mass_ratio * 1000.0);
    
    if mass_classification == 2u {
        state.total_mass = 1.0;
    }
}