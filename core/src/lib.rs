//! Aetherion Core - Administrative Field System
//! 
//! This module provides the hierarchical administrative structure that governs
//! all physical assets in the simulation: County → City → Borough → Neighborhood → Parcel

use serde::{Deserialize, Serialize};
use geo::{MultiPolygon, Polygon, Point, Contains};
use std::collections::HashMap;

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcePool {
    pub population: u64,
    pub tax_revenue: f64,
    pub water_demand: f64,
    pub energy_consumption: f64,
    pub noise_index: f32,
    pub traffic_volume: f32,
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

    /// Aggregate resources from children to parents
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
                            parent.resource_pool.tax_revenue += resources.tax_revenue;
                            parent.resource_pool.water_demand += resources.water_demand;
                            parent.resource_pool.energy_consumption += resources.energy_consumption;
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
}
