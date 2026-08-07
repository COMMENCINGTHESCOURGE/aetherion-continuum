//! Aetherion Continuum - Field-Native Planetary Simulation Engine
//! 
//! A high-performance simulation engine for climate modeling, digital twins,
//! and enterprise-scale urban simulations with jurisdictional awareness.

pub mod admin_tree;

#[cfg(test)]
mod admin_tree_tests;

pub use admin_tree::{
    AdminLevel,
    AdministrativeNode,
    AdminTree,
    RuleStack,
    Restriction,
    ResourcePool,
    CommunityProfile,
    AestheticProfile,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export common types
pub use geo::{Point, MultiPolygon};
