//! pillar_traits.rs
//! Trait definitions enforcing ORDER across the 5 pillars.
//! GOVERNOR: consumed by sub_dispatch.rs + conservation_proof.rs
use crate::field::Tensor6;

pub trait Pillar {
    fn name(&self) -> &'static str;
    /// Execute one work-unit across field buffers.
    /// FIRST arg is mutable output, SECOND is input (read-only view).
    fn execute(&self, out: &mut [Tensor6], inp: &[Tensor6]);
}

pub enum PillarId {
    FieldIngestion = 0u32,
    ContinuityResolution = 1u32,
    VolumetricSimulation = 2u32,
    CohesionQEFMeshing = 3u32,
    PlanetaryCulling = 4u32,
}

impl PillarId {
    pub const COUNT: usize = 5;

    pub fn as_usize(self) -> usize { self as usize }

    pub fn validate_order(seq: &[Self]) -> bool {
        seq.windows(2).all(|w| (w[0] as u32) <= (w[1] as u32))
    }
}
