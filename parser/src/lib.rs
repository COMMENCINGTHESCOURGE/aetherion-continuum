//! OSM Boundary Parser for Administrative Hierarchy
//! 
//! Parses OpenStreetMap .osm.pbf files to extract administrative boundaries
//! (County, City, Borough, Neighborhood) and build the AdminTree

use aetherion_core::{AdminTree, AdministrativeNode, AdminLevel, RuleStack, Restriction};
use osmpbf::{Element, ElementReader, Relation, Way};
use geo::{MultiPolygon, Polygon, Point, Coordinate};
use std::collections::{HashMap, HashSet};
use rstar::RTree;

/// Spatial index entry for efficient point-in-polygon queries
struct SpatialEntry {
    centroid: Point<f64>,
    id: String,
    level_priority: i32,
}

impl rstar::PointDistance for SpatialEntry {
    fn distance_2(&self, other: &Self) -> f64 {
        let dx = self.centroid.x() - other.centroid.x();
        let dy = self.centroid.y() - other.centroid.y();
        dx * dx + dy * dy
    }

    fn contains_point_2(&self, point: &(f64, f64)) -> bool {
        // Simplified: just check distance to centroid
        let dx = self.centroid.x() - point.0;
        let dy = self.centroid.y() - point.1;
        dx * dx + dy * dy < 0.01 // Small threshold
    }
}

/// Parser for OSM administrative boundaries
pub struct BoundaryParser {
    nodes: HashMap<i64, (f64, f64)>, // Node ID -> (lat, lon)
    ways: HashMap<i64, Vec<i64>>,    // Way ID -> node IDs
    relations: Vec<ParsedRelation>,
}

struct ParsedRelation {
    id: i64,
    name: String,
    level: AdminLevel,
    member_way_ids: Vec<i64>,
    tags: HashMap<String, String>,
}

impl BoundaryParser {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            relations: vec![],
        }
    }

    /// Parse an OSM PBF file and return an AdminTree
    pub fn parse_pbf(&mut self, path: &str) -> Result<AdminTree, String> {
        let reader = ElementReader::from_path(path)
            .map_err(|e| format!("Failed to open PBF file: {}", e))?;

        // First pass: collect all nodes, ways, and relations
        reader.for_each(|element| {
            match element {
                Element::Node(node) => {
                    self.nodes.insert(node.id, (node.lat(), node.lon()));
                }
                Element::Way(way) => {
                    self.ways.insert(way.id, way.nodes);
                }
                Element::Relation(rel) => {
                    if let Some(boundary) = rel.tags.get("boundary") {
                        if boundary == "administrative" {
                            if let Some(level_str) = rel.tags.get("admin_level") {
                                let level = self.parse_admin_level(level_str);
                                let name = rel.tags.get("name")
                                    .cloned()
                                    .unwrap_or_else(|| format!("Unnamed_{}", rel.id));
                                
                                let member_ids: Vec<i64> = rel.members
                                    .iter()
                                    .filter(|m| m.role == "outer" || m.role == "")
                                    .filter_map(|m| {
                                        if let osmpbf::RelationMember::Way(way_id) = m.member {
                                            Some(way_id)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                self.relations.push(ParsedRelation {
                                    id: rel.id,
                                    name,
                                    level,
                                    member_way_ids: member_ids,
                                    tags: rel.tags.clone(),
                                });
                            }
                        }
                    }
                    
                    // Also check for neighborhoods (place=neighbourhood)
                    if rel.tags.get("place") == Some(&"neighbourhood".to_string()) {
                        let name = rel.tags.get("name")
                            .cloned()
                            .unwrap_or_else(|| format!("Neighborhood_{}", rel.id));
                        
                        let member_ids: Vec<i64> = rel.members
                            .iter()
                            .filter_map(|m| {
                                if let osmpbf::RelationMember::Way(way_id) = m.member {
                                    Some(way_id)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        self.relations.push(ParsedRelation {
                            id: rel.id,
                            name,
                            level: AdminLevel::Neighborhood,
                            member_way_ids: member_ids,
                            tags: rel.tags.clone(),
                        });
                    }
                }
                _ => {}
            }
        });

        // Build the AdminTree
        Ok(self.build_tree())
    }

    fn parse_admin_level(&self, level_str: &str) -> AdminLevel {
        match level_str {
            "6" => AdminLevel::County,
            "8" => AdminLevel::City,
            "9" => AdminLevel::Borough,
            "10" | "11" => AdminLevel::Neighborhood,
            _ => AdminLevel::City, // Default fallback
        }
    }

    /// Build polygon from way nodes
    fn build_polygon(&self, way_id: &i64) -> Option<Polygon<f64>> {
        let node_ids = self.ways.get(way_id)?;
        
        let coords: Vec<Coordinate<f64>> = node_ids
            .iter()
            .filter_map(|nid| {
                self.nodes.get(nid).map(|(lat, lon)| {
                    // Convert lat/lon to coordinates (simplified, should use proper projection)
                    Coordinate { x: *lon, y: *lat }
                })
            })
            .collect();

        if coords.len() < 3 {
            return None;
        }

        // Close the ring if not already closed
        let mut closed_coords = coords.clone();
        if coords.first() != coords.last() {
            if let Some(first) = coords.first() {
                closed_coords.push(*first);
            }
        }

        Some(Polygon::new(closed_coords.into(), vec![]))
    }

    /// Build multipolygon from relation members
    fn build_multipolygon(&self, relation: &ParsedRelation) -> Option<MultiPolygon<f64>> {
        let mut polygons = vec![];

        for way_id in &relation.member_way_ids {
            if let Some(polygon) = self.build_polygon(way_id) {
                polygons.push(polygon);
            }
        }

        if polygons.is_empty() {
            None
        } else {
            Some(MultiPolygon::new(polygons))
        }
    }

    /// Build the complete AdminTree from parsed data
    fn build_tree(&self) -> AdminTree {
        let mut tree = AdminTree::new();
        let mut spatial_index: RTree<SpatialEntry> = RTree::new();

        // Create all administrative nodes
        for relation in &self.relations {
            if let Some(multipolygon) = self.build_multipolygon(relation) {
                let id = format!("rel_{}", relation.id);
                let mut node = AdministrativeNode::new(
                    id.clone(),
                    relation.name.clone(),
                    relation.level.clone(),
                );
                node.boundary = multipolygon;
                node.osm_relation_id = Some(relation.id);

                // Calculate centroid for spatial indexing
                if let Some(centroid) = node.boundary.centroid() {
                    let entry = SpatialEntry {
                        centroid,
                        id: id.clone(),
                        level_priority: node.level.priority() as i32,
                    };
                    spatial_index.insert(entry);
                }

                tree.add_node(node);
            }
        }

        // Establish parent-child relationships via spatial containment
        // (Simplified: in production, use proper polygon containment checks)
        let node_ids: Vec<String> = tree.nodes.keys().cloned().collect();
        
        for id in &node_ids {
            if let Some(node) = tree.get_node(id) {
                if let Some(centroid) = node.boundary.centroid() {
                    // Find potential parents (higher level nodes containing this centroid)
                    let current_level = node.level.clone();
                    
                    for other in tree.nodes.values() {
                        if other.level.priority() < current_level.priority() {
                            // Check if other node contains this node's centroid
                            if other.contains_point(&centroid) {
                                if let Some(child_node) = tree.get_node_mut(id) {
                                    child_node.set_parent(other.id.clone());
                                }
                                if let Some(parent_node) = tree.get_node_mut(&other.id) {
                                    parent_node.add_child(id.clone());
                                }
                                break; // Found the immediate parent
                            }
                        }
                    }
                }
            }
        }

        tree
    }
}

/// Parse neighborhoods from OSM ways (alternative to relations)
pub fn parse_neighborhoods_from_ways(path: &str) -> Result<Vec<AdministrativeNode>, String> {
    let reader = ElementReader::from_path(path)
        .map_err(|e| format!("Failed to open PBF file: {}", e))?;
    
    let mut neighborhoods = vec![];
    let mut nodes: HashMap<i64, (f64, f64)> = HashMap::new();
    let mut neighborhood_ways: Vec<Way> = vec![];

    reader.for_each(|element| {
        match element {
            Element::Node(node) => {
                nodes.insert(node.id, (node.lat(), node.lon()));
            }
            Element::Way(way) => {
                if way.tags.get("place") == Some(&"neighbourhood".to_string()) ||
                   way.tags.get("boundary") == Some(&"locality".to_string()) {
                    neighborhood_ways.push(way);
                }
            }
            _ => {}
        }
    });

    for way in neighborhood_ways {
        let coords: Vec<Coordinate<f64>> = way.nodes
            .iter()
            .filter_map(|nid| {
                nodes.get(nid).map(|(lat, lon)| {
                    Coordinate { x: *lon, y: *lat }
                })
            })
            .collect();

        if coords.len() >= 3 {
            let mut closed = coords.clone();
            if coords.first() != coords.last() {
                if let Some(first) = coords.first() {
                    closed.push(*first);
                }
            }

            let polygon = Polygon::new(closed.into(), vec![]);
            let multipolygon = MultiPolygon::new(vec![polygon]);
            
            let name = way.tags.get("name")
                .cloned()
                .unwrap_or_else(|| format!("Neighborhood_{}", way.id));

            let mut node = AdministrativeNode::new(
                format!("way_{}", way.id),
                name,
                AdminLevel::Neighborhood,
            );
            node.boundary = multipolygon;
            node.osm_way_id = Some(way.id);

            neighborhoods.push(node);
        }
    }

    Ok(neighborhoods)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = BoundaryParser::new();
        assert!(parser.nodes.is_empty());
        assert!(parser.ways.is_empty());
        assert!(parser.relations.is_empty());
    }

    #[test]
    fn test_admin_level_parsing() {
        let parser = BoundaryParser::new();
        assert_eq!(parser.parse_admin_level("6"), AdminLevel::County);
        assert_eq!(parser.parse_admin_level("8"), AdminLevel::City);
        assert_eq!(parser.parse_admin_level("9"), AdminLevel::Borough);
        assert_eq!(parser.parse_admin_level("10"), AdminLevel::Neighborhood);
    }
}
