//! Aetherion Bridge - GDExtension for Godot 4
//! 
//! This module provides the Rust-to-Godot bridge, exposing the AdminTree
//! and simulation engine to GDScript via GDExtension.

#[cfg(feature = "godot-extension")]
mod godot_integration;

#[cfg(feature = "godot-extension")]
pub use godot_integration::*;

use aetherion_core::{AdminTree, AdministrativeNode, AdminLevel, Point};
use serde_json;
use std::sync::{Arc, Mutex};

/// Main bridge struct that Godot will interact with
pub struct AetherionBridge {
    admin_tree: Arc<Mutex<AdminTree>>,
}

impl AetherionBridge {
    pub fn new() -> Self {
        Self {
            admin_tree: Arc::new(Mutex::new(AdminTree::new())),
        }
    }

    /// Load AdminTree from JSON file
    pub fn load_from_json(&self, path: &str) -> Result<(), String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let tree: AdminTree = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        *self.admin_tree.lock().unwrap() = tree;
        Ok(())
    }

    /// Save AdminTree to JSON file
    pub fn save_to_json(&self, path: &str) -> Result<(), String> {
        let tree = self.admin_tree.lock().unwrap();
        let json = serde_json::to_string_pretty(&*tree)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    /// Find administrative node at given coordinates (lat, lon)
    pub fn find_node_at_coords(&self, lat: f64, lon: f64) -> Option<String> {
        let tree = self.admin_tree.lock().unwrap();
        let point = Point::new(lon, lat); // Note: Geo uses (x=lon, y=lat)
        
        tree.find_node_at_point(&point).map(|n| n.id.clone())
    }

    /// Get node details as JSON string
    pub fn get_node_details(&self, node_id: &str) -> Option<String> {
        let tree = self.admin_tree.lock().unwrap();
        let node = tree.get_node(node_id)?;
        
        serde_json::to_string(node).ok()
    }

    /// Get resolved rules at coordinates
    pub fn get_rules_at_coords(&self, lat: f64, lon: f64) -> String {
        let tree = self.admin_tree.lock().unwrap();
        let point = Point::new(lon, lat);
        let rules = tree.resolve_rules(&point);
        
        serde_json::to_string(&rules).unwrap_or_else(|_| "{}".to_string())
    }

    /// Add a custom restriction to a node
    pub fn add_restriction(&self, node_id: &str, key: &str, value: &str, priority: u8) -> bool {
        let mut tree = self.admin_tree.lock().unwrap();
        
        if let Some(node) = tree.get_node_mut(node_id) {
            use aetherion_core::Restriction;
            let restriction = Restriction {
                key: key.to_string(),
                value: value.to_string(),
                priority,
                time_of_day: None,
            };
            node.rule_stack.apply_override(restriction);
            true
        } else {
            false
        }
    }

    /// Get all child nodes of a given node
    pub fn get_children(&self, parent_id: &str) -> Vec<String> {
        let tree = self.admin_tree.lock().unwrap();
        
        tree.get_node(parent_id)
            .map(|n| n.child_ids.clone())
            .unwrap_or_default()
    }

    /// Get parent node ID
    pub fn get_parent(&self, node_id: &str) -> Option<String> {
        let tree = self.admin_tree.lock().unwrap();
        tree.get_node(node_id).and_then(|n| n.parent_id.clone())
    }

    /// Aggregate resources up the hierarchy
    pub fn aggregate_resources(&self) {
        let mut tree = self.admin_tree.lock().unwrap();
        tree.aggregate_resources();
    }

    /// Update resource pool for a node
    pub fn update_resources(&self, node_id: &str, population: u64, tax: f64, noise: f32) -> bool {
        let mut tree = self.admin_tree.lock().unwrap();
        
        if let Some(node) = tree.get_node_mut(node_id) {
            node.resource_pool.population = population;
            node.resource_pool.tax_revenue = tax;
            node.resource_pool.noise_index = noise;
            true
        } else {
            false
        }
    }

    /// Get community profile as JSON
    pub fn get_community_profile(&self, node_id: &str) -> Option<String> {
        let tree = self.admin_tree.lock().unwrap();
        let node = tree.get_node(node_id)?;
        
        serde_json::to_string(&node.community_profile).ok()
    }

    /// Set community profile from JSON
    pub fn set_community_profile(&self, node_id: &str, profile_json: &str) -> bool {
        let mut tree = self.admin_tree.lock().unwrap();
        
        if let Some(node) = tree.get_node_mut(node_id) {
            if let Ok(profile) = serde_json::from_str(profile_json) {
                node.community_profile = profile;
                return true;
            }
        }
        false
    }

    /// Get aesthetic profile for procedural generation
    pub fn get_aesthetic_profile(&self, node_id: &str) -> Option<String> {
        let tree = self.admin_tree.lock().unwrap();
        let node = tree.get_node(node_id)?;
        
        serde_json::to_string(&node.aesthetic_profile).ok()
    }

    /// Count nodes by level
    pub fn count_by_level(&self) -> serde_json::Value {
        let tree = self.admin_tree.lock().unwrap();
        let mut counts = serde_json::Map::new();
        
        for node in tree.nodes.values() {
            let level_name = match node.level {
                AdminLevel::County => "county",
                AdminLevel::City => "city",
                AdminLevel::Borough => "borough",
                AdminLevel::Neighborhood => "neighborhood",
                AdminLevel::Parcel => "parcel",
            };
            
            let count = counts.get(level_name).and_then(|v| v.as_u64()).unwrap_or(0);
            counts.insert(level_name.to_string(), serde_json::json!(count + 1));
        }
        
        serde_json::Value::Object(counts)
    }
}

impl Default for AetherionBridge {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe singleton access
lazy_static::lazy_static! {
    static ref BRIDGE_INSTANCE: Arc<Mutex<AetherionBridge>> = 
        Arc::new(Mutex::new(AetherionBridge::new()));
}

pub fn get_bridge() -> Arc<Mutex<AetherionBridge>> {
    BRIDGE_INSTANCE.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = AetherionBridge::new();
        assert!(bridge.admin_tree.lock().unwrap().nodes.is_empty());
    }

    #[test]
    fn test_find_node_empty() {
        let bridge = AetherionBridge::new();
        assert!(bridge.find_node_at_coords(47.6062, -122.3321).is_none());
    }
}
