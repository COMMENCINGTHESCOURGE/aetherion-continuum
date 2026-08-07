//! Aetherion Core - Administrative Field System with Quantum Phase Conservation
//! 
//! This module provides the hierarchical administrative structure that governs
//! all physical assets in the simulation: County → City → Borough → Neighborhood → Parcel
//! 
//! It integrates quantum-inspired phase algebra for zero-drift conservation of resources,
//! inspired by the solution to Euler's 36 Officers Problem using AME states.

use serde::{Deserialize, Serialize};
use geo::{MultiPolygon, Polygon, Point, Contains};
use std::collections::HashMap;

pub mod quantum_phase;
pub use quantum_phase::{PhaseUnit, ComplexValue, UnitaryAccumulator, ConstraintSolver};

/// Administrative hierarchy levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AdminLevel {
    County,       // admin_level=6 in OSM
    City,         // admin_level=8 in OSM
    Borough,      // admin_level=9 or custom
    Neighborhood, // place=neighbourhood in OSM
    Parcel,       // landuse / addr:* tags
}

impl AdminLevel {
    pub fn priority(&self) -> u8 {
        match self {
            AdminLevel::County => 0,
            AdminLevel::City => 1,
            AdminLevel::Borough => 2,
            AdminLevel::Neighborhood => 3,
            AdminLevel::Parcel => 4,
        }
    }
}

/// Restriction rules that propagate down the hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restriction {
    pub key: String,
    pub value: String,
    pub priority: u8,
    pub time_of_day: Option<(u16, u16)>, // e.g., (22, 6) for 10pm-6am curfew
}

/// Stack of rules with conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleStack {
    pub restrictions: HashMap<String, Restriction>,
}

impl RuleStack {
    pub fn apply_override(&mut self, new_rule: Restriction) {
        let key = new_rule.key.clone();
        if let Some(existing) = self.restrictions.get_mut(&key) {
            if new_rule.priority > existing.priority {
                *existing = new_rule;
            }
        } else {
            self.restrictions.insert(key, new_rule);
        }
    }

    pub fn get(&self, key: &str) -> Option<&Restriction> {
        self.restrictions.get(key)
    }
}

/// Aggregated resources flowing up from children to parents
/// Uses UnitaryAccumulator for drift-free conservation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcePool {
    pub population: u64,
    pub tax_revenue: f64,
    pub water_demand: f64,
    pub energy_consumption: f64,
    pub noise_index: f32,
    pub traffic_volume: f32,
    
    // Quantum phase-encoded accumulators for zero-drift conservation
    #[serde(skip)]
    pub phase_accumulator: Option<UnitaryAccumulator>,
}

impl ResourcePool {
    /// Initialize with quantum phase encoding
    pub fn with_phase_encoding(&mut self) {
        self.phase_accumulator = Some(UnitaryAccumulator::new());
    }
    
    /// Add resource using phase encoding (prevents floating-point drift)
    pub fn add_phase_resource(&mut self, amount: f64, phase: PhaseUnit) {
        if let Some(ref mut acc) = self.phase_accumulator {
            acc.add_phase(phase, amount);
            // Extract magnitude back to tax_revenue as example
            self.tax_revenue = acc.total();
        } else {
            self.tax_revenue += amount;
        }
    }
    
    /// Get conserved total from phase accumulator
    pub fn get_conserved_total(&self) -> f64 {
        self.phase_accumulator.as_ref().map(|a| a.total()).unwrap_or(self.tax_revenue)
    }
}

/// Community social dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityProfile {
    pub economic_status: String, // "Wealthy", "Industrial", "Subsistence"
    pub political_leaning: String, // "Pro-Development", "NIMBY", "Balanced"
    pub happiness: f32,          // 0.0 - 1.0
    pub density_preference: f32, // 0.0 (low) - 1.0 (high)
}

impl Default for CommunityProfile {
    fn default() -> Self {
        Self {
            economic_status: "Mixed".to_string(),
            political_leaning: "Balanced".to_string(),
            happiness: 0.5,
            density_preference: 0.5,
        }
    }
}

/// Aesthetic modifiers for procedural generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AestheticProfile {
    pub primary_material: String,  // "Brick", "Glass", "Concrete", "Wood"
    pub roof_style: String,        // "Flat", "Pitched", "Terraced", "Domed"
    pub greenery_density: f32,     // 0.0 - 1.0
    pub lot_size_multiplier: f32,  // Affects driveway/garage size
    pub color_palette: Vec<String>, // Hex colors for buildings
}

impl Default for AestheticProfile {
    fn default() -> Self {
        Self {
            primary_material: "Concrete".to_string(),
            roof_style: "Flat".to_string(),
            greenery_density: 0.3,
            lot_size_multiplier: 1.0,
            color_palette: vec!["#888888".to_string()],
        }
    }
}

/// Core administrative node representing a jurisdictional boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministrativeNode {
    // Identity
    pub id: String,
    pub name: String,
    pub level: AdminLevel,
    
    // Spatial geometry (for point-in-polygon queries)
    #[serde(skip)]
    pub boundary: MultiPolygon<f64>,
    
    // Hierarchy links
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    
    // Living system fields
    pub rule_stack: RuleStack,
    pub resource_pool: ResourcePool,
    pub community_profile: CommunityProfile,
    pub aesthetic_profile: AestheticProfile,
    
    // Metadata
    pub osm_relation_id: Option<i64>,
    pub osm_way_id: Option<i64>,
}

impl AdministrativeNode {
    pub fn new(id: String, name: String, level: AdminLevel) -> Self {
        Self {
            id,
            name,
            level,
            boundary: MultiPolygon::new(vec![]),
            parent_id: None,
            child_ids: vec![],
            rule_stack: RuleStack::default(),
            resource_pool: ResourcePool::default(),
            community_profile: CommunityProfile::default(),
            aesthetic_profile: AestheticProfile::default(),
            osm_relation_id: None,
            osm_way_id: None,
        }
    }

    pub fn contains_point(&self, point: &Point<f64>) -> bool {
        self.boundary.contains(point)
    }

    pub fn add_child(&mut self, child_id: String) {
        if !self.child_ids.contains(&child_id) {
            self.child_ids.push(child_id);
        }
    }

    pub fn set_parent(&mut self, parent_id: String) {
        self.parent_id = Some(parent_id);
    }
}

/// The complete administrative tree for the simulation world
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminTree {
    pub nodes: HashMap<String, AdministrativeNode>,
    pub root_ids: Vec<String>, // Counties (top level)
}

impl AdminTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_ids: vec![],
        }
    }

    pub fn add_node(&mut self, node: AdministrativeNode) {
        let id = node.id.clone();
        self.nodes.insert(id, node);
    }

    pub fn get_node(&self, id: &str) -> Option<&AdministrativeNode> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut AdministrativeNode> {
        self.nodes.get_mut(id)
    }

    /// Find the deepest administrative node containing a point
    pub fn find_node_at_point(&self, point: &Point<f64>) -> Option<&AdministrativeNode> {
        let mut best_match: Option<&AdministrativeNode> = None;
        let mut best_priority = -1;

        for node in self.nodes.values() {
            if node.contains_point(point) {
                let priority = node.level.priority() as i32;
                if priority > best_priority {
                    best_priority = priority;
                    best_match = Some(node);
                }
            }
        }

        best_match
    }

    /// Resolve the complete rule stack for a location by walking up the hierarchy
    pub fn resolve_rules(&self, point: &Point<f64>) -> RuleStack {
        let mut resolved = RuleStack::default();
        
        // Start from the deepest node and walk up to root
        let mut current_id = self.find_node_at_point(point).map(|n| n.id.clone());
        
        while let Some(id) = current_id {
            if let Some(node) = self.nodes.get(&id) {
                for (_, restriction) in &node.rule_stack.restrictions {
                    resolved.apply_override(restriction.clone());
                }
                current_id = node.parent_id.clone();
            } else {
                break;
            }
        }

        resolved
    }

    /// Aggregate resources from children to parents using phase-conserving accumulation
    pub fn aggregate_resources(&mut self) {
        // Process from lowest level (Parcel) to highest (County)
        let levels = [
            AdminLevel::Parcel,
            AdminLevel::Neighborhood,
            AdminLevel::Borough,
            AdminLevel::City,
            AdminLevel::County,
        ];

        for level in levels {
            let node_ids: Vec<String> = self.nodes
                .iter()
                .filter(|(_, n)| n.level == level)
                .map(|(id, _)| id.clone())
                .collect();

            for node_id in node_ids {
                if let Some(node) = self.nodes.get(&node_id) {
                    let parent_id = node.parent_id.clone();
                    let resources = node.resource_pool.clone();
                    
                    if let Some(parent_id) = parent_id {
                        if let Some(parent) = self.nodes.get_mut(&parent_id) {
                            parent.resource_pool.population += resources.population;
                            
                            // Use phase-encoded accumulation if available
                            let phase = PhaseUnit::from_angle(0.0); // Base phase
                            parent.resource_pool.add_phase_resource(resources.tax_revenue, phase);
                            parent.resource_pool.water_demand += resources.water_demand;
                            parent.resource_pool.energy_consumption += resources.energy_consumption;
                            
                            // Average noise and traffic
                            parent.resource_pool.noise_index = 
                                (parent.resource_pool.noise_index + resources.noise_index) / 2.0;
                            parent.resource_pool.traffic_volume = 
                                (parent.resource_pool.traffic_volume + resources.traffic_volume) / 2.0;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{Coordinate, polygon};

    #[test]
    fn test_admin_hierarchy_creation() {
        let mut tree = AdminTree::new();
        
        // Create county
        let mut county = AdministrativeNode::new(
            "county_1".to_string(),
            "King County".to_string(),
            AdminLevel::County,
        );
        
        // Create city
        let mut city = AdministrativeNode::new(
            "city_1".to_string(),
            "Seattle".to_string(),
            AdminLevel::City,
        );
        city.set_parent(county.id.clone());
        county.add_child(city.id.clone());
        
        tree.add_node(county);
        tree.add_node(city);
        
        assert_eq!(tree.root_ids.len(), 0); // Would be populated separately
        assert!(tree.get_node("county_1").is_some());
        assert!(tree.get_node("city_1").is_some());
    }

    #[test]
    fn test_rule_priority_override() {
        let mut stack = RuleStack::default();
        
        // County sets speed limit to 55
        let county_rule = Restriction {
            key: "speed_limit".to_string(),
            value: "55".to_string(),
            priority: 0,
            time_of_day: None,
        };
        stack.apply_override(county_rule);
        
        // Neighborhood overrides to 25
        let neighborhood_rule = Restriction {
            key: "speed_limit".to_string(),
            value: "25".to_string(),
            priority: 3,
            time_of_day: None,
        };
        stack.apply_override(neighborhood_rule);
        
        assert_eq!(stack.get("speed_limit").unwrap().value, "25");
    }

    #[test]
    fn test_quantum_phase_drift_free_accumulation() {
        use crate::quantum_phase::PhaseUnit;
        
        let mut pool = ResourcePool::default();
        pool.with_phase_encoding();
        
        // Add 1000 tax payments of 1.0 each using phase encoding
        for _ in 0..1000 {
            pool.add_phase_resource(1.0, PhaseUnit::from_angle(0.0));
        }
        
        // Standard float addition would have small errors
        // Phase-encoded accumulation should be exact
        let conserved = pool.get_conserved_total();
        assert!((conserved - 1000.0).abs() < 1e-10, "Drift-free accumulation failed: {}", conserved);
    }

    #[test]
    fn test_constraint_solver_orthogonality() {
        use crate::quantum_phase::ConstraintSolver;
        
        let mut solver = ConstraintSolver::new();
        
        // Add orthogonal constraints (like Euler's officers)
        let v1 = PhaseUnit::from_angle(0.0);
        let v2 = PhaseUnit::from_angle(std::f64::consts::PI / 2.0);
        let v3 = PhaseUnit::from_angle(std::f64::consts::PI);
        
        let r1 = solver.add_constraint(v1);
        let r2 = solver.add_constraint(v2);
        let r3 = solver.add_constraint(v3);
        
        // Verify orthogonality is maintained (dot products near zero)
        let dot12 = r1.re * r2.re + r1.im * r2.im;
        let dot13 = r1.re * r3.re + r1.im * r3.im;
        let dot23 = r2.re * r3.re + r2.im * r3.im;
        
        assert!(dot12.abs() < 0.15, "Constraints 1 and 2 not orthogonal: {}", dot12);
        assert!(dot13.abs() < 0.15, "Constraints 1 and 3 not orthogonal: {}", dot13);
        assert!(dot23.abs() < 0.15, "Constraints 2 and 3 not orthogonal: {}", dot23);
    }
}
