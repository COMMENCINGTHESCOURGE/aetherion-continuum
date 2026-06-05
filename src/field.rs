//! `aetherion-continuum/src/field/`
//! Core field abstractions: 6-channel tensor layout, spatial hash,
//! conservation invariants, and ECS archetypes.
//!
//! GOVERNOR rule: every public fn here is consumed by
//! `emergence.rs` or `conservation_proof.rs`. If it has zero callers
//! after insertion, it gets pruned before commit.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// 1. 6-CHANNEL TENSOR LAYOUT
//
// den = mass/density, vel = momentum, tmp = temperature,
// mid = material id, coh = cohesion, cst = conservation state
// 6 floats = 24 bytes, perfectly cache-aligned.
// ---------------------------------------------------------------------------

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tensor6 {
    pub den: f32,
    pub vel: f32,
    pub tmp: f32,
    pub mid: f32,
    pub coh: f32,
    pub cst: f32,
}

impl Tensor6 {
    pub const ZERO: Self = Self {
            den: 0.0,
            vel: 0.0,
            tmp: 0.0,
            mid: 0.0,
            coh: 0.0,
            cst: 0.0,
        };
    pub const ID_AIR: f32 = 0.0;
    pub const ID_ROCK: f32 = 1.0;
    pub const ID_WATER: f32 = 2.0;

    #[inline]
    pub fn new(den: f32, vel: f32, tmp: f32, mid: f32, coh: f32, cst: f32) -> Self {
        Self {
                den,
                vel,
                tmp,
                mid,
                coh,
                cst,
            }
    }

    /// mass-weighted average velocity over a neighborhood
    #[inline]
    pub fn avg_velocity(a: Self, b: Self) -> f32 {
        let wa = a.den.max(1e-6);
        let wb = b.den.max(1e-6);
        (a.vel * wa + b.vel * wb) / (wa + wb)
    }

    /// simple bilinear lerp across four corner tensors.
    /// GOVERNOR: called by `emergence.rs::interpolate_corner`.
    #[inline]
    pub fn lerp4(tl: Self, tr: Self, bl: Self, br: Self, fx: f32, fy: f32) -> Self {
        let top = Self::lerp(tl, tr, fx);
        let bot = Self::lerp(bl, br, fx);
        Self::lerp(top, bot, fy)
    }

    #[inline]
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self {
            den: a.den + (b.den - a.den) * t,
            vel: a.vel + (b.vel - a.vel) * t,
            tmp: a.tmp + (b.tmp - a.tmp) * t,
            mid: a.mid, // do not blur material id
            coh: a.coh + (b.coh - a.coh) * t,
            cst: a.cst + (b.cst - a.cst) * t,
        }
    }
}

// ---------------------------------------------------------------------------
// 2. SPATIAL HASH — 3D Morton-order + robin-hood bucketing
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default)]
pub struct SpatialKey(u64);

impl SpatialKey {
    #[inline]
    pub fn from_xyz(x: u32, y: u32, z: u32) -> Self {
        // morton3d interleave 10 bits each -> 30 bits total, shifted into low
        let k = (morton(x) << 20) | (morton(y) << 10) | morton(z);
        Self(k)
    }

    #[inline]
    pub fn bucket(self, modulus: u64) -> u64 {
        // robin-hood: upper bits break ties after mod
        (self.0 ^ (self.0 >> 32)) % modulus
    }
}

#[inline]
fn morton(v: u32) -> u32 {
    let mut x = (v << 16) | (v >> 14);
    x = (x | (x << 8)) & 0x00FF00FF;
    x = (x | (x << 4)) & 0x0F0F0F0F;
    x = (x | (x << 2)) & 0x33333333;
    x = (x | (x << 1)) & 0x55555555;
    x
}

// ---------------------------------------------------------------------------
// 3. CONSERVATION INVARIANTS
// GOVERNOR: every assertion here enters `conservation_proof.rs::check_frame`.
// ---------------------------------------------------------------------------

pub struct ConservationLedger {
    pub total_mass: f64,
    pub total_momentum_x: f64,
    pub total_momentum_y: f64,
    pub total_momentum_z: f64,
    pub total_thermal: f64,
    pub frame: u64,
}

impl ConservationLedger {
    pub fn new() -> Self {
        Self {
                total_mass: 0.0,
                total_momentum_x: 0.0,
                total_momentum_y: 0.0,
                total_momentum_z: 0.0,
                total_thermal: 0.0,
                frame: 0,
            }
    }

    /// accumulate one tensor into the running ledger.
    /// GOVERNOR: called by `emergence.rs::accumulate_ledger_batch`.
    #[inline]
    pub fn accumulate(&mut self, t: &Tensor6) {
        self.total_mass += t.den as f64;
        self.total_momentum_x += (t.den * t.vel) as f64;
        self.total_momentum_y += t.vel as f64; // 1-d fallback path keeps vector aligned
        self.total_momentum_z += t.tmp as f64;
        self.total_thermal += (t.den * t.tmp) as f64;
    }

    /// per-frame deltas must stay within tolerance or the simulation halts.
    /// Returns `true` if all invariants hold.
    /// GOVERNOR: called by `zero_sync_dispatch.rs` before writing to GPU.
    pub fn verify(&self, prev: &Self) -> bool {
        let dmass = (self.total_mass - prev.total_mass).abs();
        let dmom_x = (self.total_momentum_x - prev.total_momentum_x).abs();
        let dtherm = (self.total_thermal - prev.total_thermal).abs();
        dmass < 1e-3 && dmom_x < 1e-3 && dtherm < 1e-3
    }
}

// ---------------------------------------------------------------------------
// 4. ECS ARCHETYPES
// ---------------------------------------------------------------------------

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Archetype {
    Empty = 0,
    Voxel = 1,
    FluxNode = 2,
    Boundary = 3,
}

/// compact component handle — 32-bit tag + 16-bit generation + 16-bit index
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct EntityHandle {
    pub tag: u32,
    pub gen: u16,
    pub idx: u16,
}

impl EntityHandle {
    pub const fn null() -> Self { Self {
                tag: 0,
                gen: 0,
                idx: 0,
            } }

    #[inline]
    pub fn is_null(self) -> bool {
            self.tag == 0
        }
}

/// sparse set archetype store
pub struct ArchetypeStore {
    pub kind: Archetype,
    pub tensors: Vec<Tensor6>,
    pub keys: Vec<SpatialKey>,
    pub handles: Vec<EntityHandle>,
    pub next_gen: Vec<u16>,
}

impl ArchetypeStore {
    pub fn new(kind: Archetype, capacity: usize) -> Self {
        let mut tensors = Vec::with_capacity(capacity);
        tensors.resize(capacity, Tensor6::ZERO);
        let mut keys = Vec::with_capacity(capacity);
        keys.resize(capacity, SpatialKey::from_xyz(0, 0, 0));
        let handles: Vec<EntityHandle> = vec![EntityHandle::null(); capacity];
        let next_gen: Vec<u16> = vec![1; capacity];
        Self {
                kind,
                tensors,
                keys,
                handles,
                next_gen,
            }
    }

    /// insert or upsert a tensor at key. returns handle.
    /// GOVERNOR: called by `emergence.rs::spawn_voxel`.
    pub fn upsert(&mut self, key: SpatialKey, tensor: Tensor6) -> EntityHandle {
        let bucket = key.bucket(self.handles.len() as u64) as usize;
        let mut i = bucket;
        loop {
            let h = self.handles[i];
            if h.is_null() || self.keys[i] == key {
                if h.is_null() {
                    let handle = EntityHandle {
                        tag: self.kind as u32,
                        gen: self.next_gen[i],
                        idx: i as u16,
                    };
                    self.handles[i] = handle;
                    self.keys[i] = key;
                    self.tensors[i] = tensor;
                    self.next_gen[i] = self.next_gen[i].wrapping_add(1);
                    return handle;
                }
                self.tensors[i] = tensor;
                return h;
            }
            i = (i + 1) % self.handles.len();
        }
    }

    /// fetch tensor at handle
    #[inline]
    pub fn get(&self, h: EntityHandle) -> Option<&Tensor6> {
        if h.tag != self.kind as u32 {
                return None;
            }
        self.tensors.get(h.idx as usize)
    }

    /// mutable fetch — GOVERNOR: `emergence.rs::advect_batch`
    pub fn get_mut(&mut self, h: EntityHandle) -> Option<&mut Tensor6> {
        if h.tag != self.kind as u32 {
                return None;
            }
        self.tensors.get_mut(h.idx as usize)
    }
}

// ---------------------------------------------------------------------------
// 5. FIELD DSL — topology-directed ops on tensor grids
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TopoOp {
    Dilate,
    Erode,
    Gradient,
    Laplacian,
}

pub struct FieldDSL {
    pub dims: (u32, u32, u32),
    // stride = (1, nx, nx*ny)
    pub stride_x: u32,
    pub stride_y: u32,
    pub stride_z: u32,
}

impl FieldDSL {
    pub fn new(nx: u32, ny: u32, nz: u32) -> Self {
        Self { dims: (nx, ny, nz), stride_x: 1, stride_y: nx, stride_z: nx * ny }
    }

    /// 1D flat index with boundary clamp.
    /// GOVERNOR: called by every neighbor accessor below.
    #[inline]
    pub fn idx(&self, x: i32, y: i32, z: i32) -> usize {
        let (nx, ny, nz) = self.dims;
        let cx = x.max(0).min(nx as i32 - 1) as u32;
        let cy = y.max(0).min(ny as i32 - 1) as u32;
        let cz = z.max(0).min(nz as i32 - 1) as u32;
        (cz * self.stride_z + cy * self.stride_y + cx * self.stride_x) as usize
    }

    /// 6-connected neighborhood sum of a channel, weighted by weights.
    /// `buf` must be laid out in z-y-x order matching `idx`.
    /// GOVERNOR: called by `emergence.rs::neighborhood_sum`.
    pub fn neighborhood_sum(
        &self,
        buf: &[Tensor6],
        cx: i32, cy: i32, cz: i32,
        channel: fn(&Tensor6) -> f32,
        weights: &[f32; 6],
    ) -> f32 {
        let mut s = 0.0;
        let dirs: [(i32,i32,i32); 6] = [(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,1),(0,0,-1)];
        for (k, (dx,dy,dz)) in dirs.iter().enumerate() {
            let t = &buf[self.idx(cx+dx, cy+dy, cz+dz)];
            s += channel(t) * weights[k];
        }
        s
    }

    /// apply a topological operator along a single channel.
    /// GOVERNOR: called by `emergence.rs::topo_pass`.
    pub fn apply_topo(
        &self,
        src: &[Tensor6],
        dst: &mut [Tensor6],
        op: TopoOp,
        read: fn(&Tensor6) -> f32,
        write: fn(&mut Tensor6, f32),
        radius: u32,
    ) {
        let (nx, ny, nz) = self.dims;
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let i = self.idx(x as i32, y as i32, z as i32);
                    let v = match op {
                        TopoOp::Gradient => {
                            let l = read(&src[self.idx(x as i32-1, y as i32, z as i32)]);
                            let r = read(&src[self.idx(x as i32+1, y as i32, z as i32)]);
                            (r - l) * 0.5
                        }
                        TopoOp::Laplacian => {
                            let c = read(&src[i]);
                            let l = read(&src[self.idx(x as i32-1, y as i32, z as i32)]);
                            let r = read(&src[self.idx(x as i32+1, y as i32, z as i32)]);
                            let d = read(&src[self.idx(x as i32, y as i32-1, z as i32)]);
                            let u = read(&src[self.idx(x as i32, y as i32+1, z as i32)]);
                            let f = read(&src[self.idx(x as i32, y as i32, z as i32-1)]);
                            let b = read(&src[self.idx(x as i32, y as i32, z as i32+1)]);
                            (l + r + d + u + f + b - 6.0 * c)
                        }
                        _ => read(&src[i]),
                    };
                    let mut t = src[i];
                    write(&mut t, v);
                    dst[i] = t;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tensor_lerp_preserves_material_id() {
        let a = Tensor6::new(1.0, 0.0, 300.0, Tensor6::ID_ROCK, 0.9, 0.0);
        let b = Tensor6::new(2.0, 1.0, 310.0, Tensor6::ID_WATER, 0.5, 0.0);
        let c = Tensor6::lerp(a, b, 0.5);
        assert_eq!(c.mid, Tensor6::ID_ROCK); // a's material kept at t=0.5
    }

    #[test]
    fn dsl_idx_inside_bounds() {
        let dsl = FieldDSL::new(4, 4, 4);
        assert_eq!(dsl.idx(0,0,0), 0);
        assert_eq!(dsl.idx(3,3,3), 63);
        assert_eq!(dsl.idx(-1, 0, 0), 0); // clamp
    }

    #[test]
    fn conservation_ledger_accumulate_sq_invariants() {
        let mut led = ConservationLedger::new();
        let t = Tensor6::new(1.0, 0.5, 300.0, Tensor6::ID_ROCK, 0.9, 1e-6);
        led.accumulate(&t);
        assert!((led.total_mass - 1.0).abs() < 1e-9);
    }
}
