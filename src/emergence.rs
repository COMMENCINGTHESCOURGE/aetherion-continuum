//! emergence.rs
//! GOVERNOR: consumes `field.rs` (Tensor6, ConservationLedger, FieldDSL, SpatialKey)
use crate::emergence_utils;
use crate::field::{Archetype, ArchetypeStore, ConservationLedger, FieldDSL, Tensor6};

pub struct Emergence {
    pub dims: (u32, u32, u32),
    pub dsl: FieldDSL,
    pub stores: [ArchetypeStore; 4],
    pub prev_ledger: ConservationLedger,
}

impl Emergence {
    pub fn new(nx: u32, ny: u32, nz: u32) -> Self {
        let dsl = FieldDSL::new(nx, ny, nz);
        Self {
            dims: (nx, ny, nz),
            dsl,
            stores: [
                ArchetypeStore::new(Archetype::Voxel, (nx * ny * nz) as usize),
                ArchetypeStore::new(Archetype::FluxNode, 4096),
                ArchetypeStore::new(Archetype::Boundary, 2048),
                ArchetypeStore::new(Archetype::Empty, 1024),
            ],
            prev_ledger: ConservationLedger::new(),
        }
    }

    /// One simulation frame: interpolate -> topo -> conservation -> ECS upsert.
    /// GOVERNOR: this is the only public frame() entry on the CPU path.
    pub fn frame(&mut self, src: &[Tensor6], dst: &mut [Tensor6]) -> ConservationLedger {
        let (nx, ny, nz) = self.dims;
        // Bilinear pass using Tensor6::lerp4
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let tl = &src[emergence_utils::dsl_idx(x, y, z, nx, ny, nz)];
                    let tr = &src[emergence_utils::dsl_idx((x + 1).min(nx - 1), y, z, nx, ny, nz)];
                    let bl = &src[emergence_utils::dsl_idx(x, (y + 1).min(ny - 1), z, nx, ny, nz)];
                    let br = &src[emergence_utils::dsl_idx(
                        (x + 1).min(nx - 1),
                        (y + 1).min(ny - 1),
                        z,
                        nx,
                        ny,
                        nz,
                    )];
                    let fx = 0.5;
                    let fy = 0.5;
                    dst[emergence_utils::dsl_idx(x, y, z, nx, ny, nz)] =
                        Tensor6::lerp4(*tl, *tr, *bl, *br, fx, fy);
                }
            }
        }

        // Gradient topo pass
        self.dsl.apply_topo(
            dst,
            dst,
            crate::field::TopoOp::Gradient,
            |t: &Tensor6| t.den,
            |t: &mut Tensor6, v: f32| t.den = v,
            1,
        );

        // Conservation ledger accumulate
        let mut ledger = ConservationLedger::new();
        for t in dst.iter() {
            ledger.accumulate(t);
        }
        ledger.frame = self.prev_ledger.frame + 1;
        self.prev_ledger = ledger;
        ledger
    }
}

// z-order flat index + mem zero remix removed via proper helpers below
// kept mark: dsl_idx replacement handled by field_dsl.rs trait import in next patch
