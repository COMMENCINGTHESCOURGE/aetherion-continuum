//! Administrative Field System Core
//! 
//! Defines the hierarchical structure of governance (County -> City -> Borough -> Neighborhood -> Parcel)
//! and the propagation of rules (downward) and resources (upward).

use geo::{MultiPolygon, Point, Contains};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// The five levels of administrative hierarchy recognized by the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdminLevel {
    County,      // admin_level=6 (Regional infrastructure, major roads)
    City,        // admin_level=8 (Municipal services, zoning)
    Borough,     // admin_level=9 (Urban character, local districts)
    Neighborhood,// place=neighbourhood (Social dynamics, HOA rules)
    Parcel,      // landuse/private (Driveways, garages, individual lots)
}

/// A restriction rule that can be enforced at any level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restriction {
    pub key: String,                  // e.g., "max_speed", "vehicle_type", "noise_curfew"
    pub value: String,                // e.g., "30_mph", "no_trucks", "22:00-06:00"
    pub priority: u8,                 // Higher priority overrides lower (Parcel > County)
    pub time_of_day: Option<(u8, u8)>,// Optional active hours (start_hour, end_hour)
}

/// The stack of rules applicable to this node, resolved from ancestors.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleStack {
    pub restrictions: HashMap<String, Restriction>,
}

impl RuleStack {
    /// Applies a new rule, respecting priority.
    pub fn apply(&mut self, new_rule: Restriction) {
        let key = new_rule.key.clone();
        if let Some(existing) = self.restrictions.get_mut(&key) {
            if new_rule.priority >= existing.priority {
                *existing = new_rule;
            }
        } else {
            self.restrictions.insert(key, new_rule);
        }
    }

    /// Merges parent rules into this stack, allowing local rules to override.
    pub fn inherit_from(&mut self, parent: &RuleStack) {
        for (key, rule) in &parent.restrictions {
            if !self.restrictions.contains_key(key) {
                self.restrictions.insert(key.clone(), rule.clone());
            }
        }
    }
}

/// Aggregated resources flowing UP from children to parents.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcePool {
    pub population: u64,
    pub tax_revenue: f64,
    pub water_demand: f64,       // Liters/day
    pub energy_consumption: f64, // kWh/day
    pub waste_production: f64,   // kg/day
    pub traffic_load: f32,       // Normalized 0.0-1.0
}

/// Social and political profile of the area (affects agent behavior).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityProfile {
    pub economic_status: String,        // "Low", "Middle", "High", "Industrial"
    pub political_leaning: String,      // "Pro-Development", "Preservationist", "NIMBY"
    pub happiness_index: f32,           // 0.0 (unrest) to 1.0 (thriving)
    pub density_preference: f32,        // 0.0 (rural) to 1.0 (urban)
}

impl Default for CommunityProfile {
    fn default() -> Self {
        Self {
            economic_status: "Middle".to_string(),
            political_leaning: "Neutral".to_string(),
            happiness_index: 0.5,
            density_preference: 0.5,
        }
    }
}

/// Visual parameters for procedural generation (Symbios-Tensor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AestheticProfile {
    pub dominant_material: String,      // "Brick", "Wood", "Concrete", "Asphalt"
    pub roof_style: String,             // "Pitched", "Flat", "Gabled"
    pub greenery_density: f32,          // 0.0 (barren) to 1.0 (forest)
    pub lot_size_avg: f32,              // Meters squared (guides driveway length)
    pub fence_style: String,            // "None", "Picket", "Chainlink", "Wall"
}

impl Default for AestheticProfile {
    fn default() -> Self {
        Self {
            dominant_material: "Generic".to_string(),
            roof_style: "Pitched".to_string(),
            greenery_density: 0.3,
            lot_size_avg: 500.0,
            fence_style: "None".to_string(),
        }
    }
}

/// The core node representing an administrative unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministrativeNode {
    // Identity
    pub id: String,                     // Unique ID (OSM Relation ID or generated)
    pub name: String,
    pub level: AdminLevel,
    
    // Geometry (The physical boundary)
    #[serde(skip)] // Skip serialization for large geometry, reload from disk/cache
    pub boundary: MultiPolygon<f64>,
    
    // Hierarchy Links
    pub parent_id: Option<String>,      // Points to the containing unit (e.g., Borough -> City)
    pub child_ids: Vec<String>,         // Points to contained units
    
    // Dynamic State
    pub rule_stack: RuleStack,
    pub resource_pool: ResourcePool,
    pub community_profile: CommunityProfile,
    pub aesthetic_profile: AestheticProfile,
}

impl AdministrativeNode {
    pub fn new(id: String, name: String, level: AdminLevel, boundary: MultiPolygon<f64>) -> Self {
        let mut node = Self {
            id,
            name,
            level,
            boundary,
            parent_id: None,
            child_ids: Vec::new(),
            rule_stack: RuleStack::default(),
            resource_pool: ResourcePool::default(),
            community_profile: CommunityProfile::default(),
            aesthetic_profile: AestheticProfile::default(),
        };
        
        // Set default rules based on level
        node.set_default_rules();
        node
    }

    fn set_default_rules(&mut self) {
        let priority = match self.level {
            AdminLevel::County => 10,
            AdminLevel::City => 20,
            AdminLevel::Borough => 30,
            AdminLevel::Neighborhood => 40,
            AdminLevel::Parcel => 50,
        };

        // Example defaults
        if self.level == AdminLevel::County {
            self.rule_stack.apply(Restriction {
                key: "max_speed_base".to_string(),
                value: "55_mph".to_string(),
                priority,
                time_of_day: None,
            });
        }
        if self.level == AdminLevel::Neighborhood {
            self.rule_stack.apply(Restriction {
                key: "noise_curfew".to_string(),
                value: "22:00-07:00".to_string(),
                priority,
                time_of_day: Some((22, 7)),
            });
        }
    }

    /// Checks if a point lies within this administrative boundary.
    pub fn contains(&self, point: &Point<f64>) -> bool {
        self.boundary.contains(point)
    }

    /// Recursively aggregates resources from children.
    pub fn aggregate_resources(&mut self, children: &[&AdministrativeNode]) {
        let mut total = ResourcePool::default();
        for child in children {
            total.population += child.resource_pool.population;
            total.tax_revenue += child.resource_pool.tax_revenue;
            total.water_demand += child.resource_pool.water_demand;
            total.energy_consumption += child.resource_pool.energy_consumption;
            total.waste_production += child.resource_pool.waste_production;
            total.traffic_load = total.traffic_load.max(child.resource_pool.traffic_load);
        }
        self.resource_pool = total;
    }
}

/// The root manager holding the entire administrative tree.
#[derive(Debug, Default)]
pub struct AdminTree {
    pub nodes: HashMap<String, AdministrativeNode>,
    pub root_ids: Vec<String>, // Top-level nodes (Counties)
}

impl AdminTree {
    pub fn add_node(&mut self, node: AdministrativeNode) {
        let id = node.id.clone();
        self.nodes.insert(id, node);
    }

    /// Resolves the full rule stack for a specific node by walking up the parent chain.
    pub fn get_effective_rules(&self, node_id: &str) -> RuleStack {
        let mut stack = RuleStack::default();
        let mut current_id = Some(node_id.to_string());

        // Walk up from leaf to root, collecting nodes
        let mut chain = Vec::new();
        while let Some(id) = current_id {
            if let Some(node) = self.nodes.get(&id) {
                chain.push(node);
                current_id = node.parent_id.clone();
            } else {
                break;
            }
        }

        // Apply from root (lowest priority) down to leaf (highest)
        for node in chain.iter().rev() {
            for (_, rule) in &node.rule_stack.restrictions {
                stack.apply(rule.clone());
            }
        }
        stack
    }

    /// Finds the deepest administrative node containing a given point.
    pub fn find_deepest_node(&self, point: &Point<f64>) -> Option<&AdministrativeNode> {
        let mut best_node: Option<&AdministrativeNode> = None;
        let mut max_depth = -1;

        for node in self.nodes.values() {
            if node.contains(point) {
                let depth = match node.level {
                    AdminLevel::County => 0,
                    AdminLevel::City => 1,
                    AdminLevel::Borough => 2,
                    AdminLevel::Neighborhood => 3,
                    AdminLevel::Parcel => 4,
                };
                if depth > max_depth {
                    max_depth = depth;
                    best_node = Some(node);
                }
            }
        }
        best_node
    }
}
