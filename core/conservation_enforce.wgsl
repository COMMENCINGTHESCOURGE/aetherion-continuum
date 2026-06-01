//   M : E : P  —  mass, energy, momentum
//   three invariants. violate one, the proof breaks.
//
//   accumulate_invariants pass reads global state for drift tracking.
//   actual atomicAdd accumulation is in field_tensor.wgsl:163-164.
//   this pass is a diagnostic readout, not a write — keep for drift monitoring.

struct ConservationState {
    mass_drift: f32,
    energy_drift: f32,
    momentum_drift: vec3<f32>,
    correction_applied: u32,
    cells_corrected: u32,
};

struct CorrectionLog {
    cell_idx: u32,
    pre_mass: f32,
    post_mass: f32,
    divergence_detected: f32,
    timestamp: f32,
};

// ═══ VINCULUM BARS ═══

const MASS_EPSILON: f32     = 1e-5;   // ¯conserved|leaking
const ENERGY_EPSILON: f32   = 1e-4;   // ¯bound|runaway
const MOMENTUM_EPSILON: f32 = 1e-4;   // ¯balanced|drifting
const MAX_CORRECTION_BAR: f32 = 0.01; // ¯safe|overcorrected — max 1% per cell per frame

const ENERGY_CAP: f32       = 1e6;    // ¯finite|divergent
const ENERGY_DAMP: f32      = 0.999;  // ¯cap→damp per excess
const GLOBAL_SCALE_BAR: f32 = 0.001;  // ¯drift→correction scale
const DRIFT_MULTIPLIER: f32 = 100.0;  // ¯local|global threshold trigger

const COHESION_SOLID_FLOOR: f32 = 0.15; // ¯phase0|valid — solid must have cohesion

// ═══ BINDINGS ═══

@group(0) @binding(0) var<storage, read_write> field: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> gradient: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> state: ConservationState;
@group(0) @binding(3) var<storage, read_write> correction_log: array<CorrectionLog>;
@group(0) @binding(4) var<storage, read_write> log_count: atomic<u32>;
@group(0) @binding(5) var<uniform> cell_count: u32;

// ═══ PASS 1: ACCUMULATE ═══

@compute @workgroup_size(256)
fn accumulate_invariants(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= cell_count { return; }

    let cell = field[idx];
    let grad = gradient[idx];

    let mass = cell.x;
    let energy = cell.x * cell.y;
    let momentum = cell.x * grad.xyz;
}

// ═══ PASS 2: ENFORCE ═══

@compute @workgroup_size(64)
fn enforce_conservation(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= cell_count { return; }

    var cell = field[idx];
    var grad = gradient[idx];

    let rho = cell.x;
    let C = cell.z;
    var correction_needed = false;
    var div: f32 = 0.0;

    if rho < 0.0 {
        cell.x = 0.0;
        correction_needed = true;
    }

    if cell.y < 0.01 && C < COHESION_SOLID_FLOOR {
        cell.z = COHESION_SOLID_FLOOR;
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
                state.mass_drift,
            );
        }
        state.correction_applied += 1u;
    }

    field[idx] = cell;
    gradient[idx] = grad;
}

// ═══ PASS 3: GLOBAL ═══

@compute @workgroup_size(1)
fn global_correction_pass() {
    if abs(state.mass_drift) > MASS_EPSILON * DRIFT_MULTIPLIER {
        let correction = 1.0 - sign(state.mass_drift) * min(abs(state.mass_drift) * GLOBAL_SCALE_BAR, MAX_CORRECTION_BAR);
        state.cells_corrected = cell_count;
    }

    if length(state.momentum_drift) > MOMENTUM_EPSILON * DRIFT_MULTIPLIER {
        state.correction_applied += 1u;
    }
}
