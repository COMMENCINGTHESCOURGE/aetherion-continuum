// ═══════════════════════════════════════════════════════════════
// AETHERION-CONTINUUM — Bridge: OpenCode Modular Interiors
// Modular interior system for vehicles and structures with field-aware
// component assembly and boolean operations
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

/// Modular interior component types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    // Structural
    FloorPanel,
    WallPanel,
    CeilingPanel,
    Bulkhead,
    
    // Functional
    Seat,
    Console,
    Storage,
    Display,
    ControlInterface,
    
    // Systems
    HVAC,
    Lighting,
    PowerConduit,
    DataConduit,
    
    // Safety
    EmergencyExit,
    FireSuppression,
    Reinforcement,
}

/// Connection interface types for modular assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    SnapFit,
    Bolted,
    Welded,
    Magnetic,
    FieldLock,  // Field-based locking mechanism
}

/// Connection point on a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoint {
    pub id: String,
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub connection_type: ConnectionType,
    pub compatible_types: Vec<ComponentType>,
}

/// Modular interior component definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModularComponent {
    pub id: String,
    pub name: String,
    pub component_type: ComponentType,
    pub scale: f32,
    pub bounds: BoundingBox,
    pub mass_kg: f32,
    pub material_id: String,
    pub connection_points: Vec<ConnectionPoint>,
    pub field_properties: ComponentFieldProperties,
    pub boolean_operations: Vec<BooleanOperation>,
    pub lod_levels: Vec<ComponentLodLevel>,
}

/// Field properties for interior components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentFieldProperties {
    pub thermal_conductivity: f32,
    pub acoustic_absorption: f32,
    pub electromagnetic_shielding: f32,
    pub structural_integrity: f32,
    pub field_coupling_strength: f32,
}

/// Boolean operation for CSG-style assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BooleanOperation {
    Union,
    Difference,
    Intersection,
    SubtractVolume(BoundingBox),
    AddVolume(BoundingBox),
}

/// Level of Detail for interior components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentLodLevel {
    pub detail_threshold: f32,
    pub triangle_count: u32,
    pub collision_complexity: CollisionComplexity,
    pub byte_offset: u64,
    pub byte_size: u64,
}

/// Collision complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollisionComplexity {
    None,           // No collision
    ConvexHull,     // Simplified convex collision
    Precise,        // Full mesh collision
    Volumetric,     // Field-aware volumetric collision
}

/// Bounding box for component geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Assembly configuration for modular interior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteriorAssembly {
    pub id: String,
    pub name: String,
    pub components: Vec<AssemblyComponent>,
    pub total_bounds: BoundingBox,
    pub total_mass_kg: f32,
    pub field_signature: FieldSignature,
}

/// Component instance in an assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyComponent {
    pub component_id: String,
    pub instance_id: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4], // Quaternion
    pub scale: f32,
    pub parent_connection: Option<String>,
    pub child_connection: Option<String>,
}

/// Overall field signature of assembled interior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSignature {
    pub thermal_emission: f32,
    pub acoustic_profile: f32,
    pub electromagnetic_signature: f32,
    pub field_distortion: f32,
}

impl ModularComponent {
    /// Create a standard floor panel component
    pub fn floor_panel_template(id: &str, width: f32, depth: f32) -> Self {
        ModularComponent {
            id: id.to_string(),
            name: format!("FloorPanel_{}x{}", width, depth),
            component_type: ComponentType::FloorPanel,
            scale: 1.0,
            bounds: BoundingBox {
                min: [-width / 2.0, -0.1, -depth / 2.0],
                max: [width / 2.0, 0.1, depth / 2.0],
            },
            mass_kg: width * depth * 5.0,
            material_id: "composite_floor_01".into(),
            connection_points: vec![
                ConnectionPoint {
                    id: "conn_front".into(),
                    position: [0.0, 0.0, depth / 2.0],
                    normal: [0.0, 0.0, 1.0],
                    connection_type: ConnectionType::SnapFit,
                    compatible_types: vec![ComponentType::FloorPanel, ComponentType::Bulkhead],
                },
                ConnectionPoint {
                    id: "conn_back".into(),
                    position: [0.0, 0.0, -depth / 2.0],
                    normal: [0.0, 0.0, -1.0],
                    connection_type: ConnectionType::SnapFit,
                    compatible_types: vec![ComponentType::FloorPanel, ComponentType::Bulkhead],
                },
                ConnectionPoint {
                    id: "conn_left".into(),
                    position: [-width / 2.0, 0.0, 0.0],
                    normal: [-1.0, 0.0, 0.0],
                    connection_type: ConnectionType::SnapFit,
                    compatible_types: vec![ComponentType::FloorPanel, ComponentType::WallPanel],
                },
                ConnectionPoint {
                    id: "conn_right".into(),
                    position: [width / 2.0, 0.0, 0.0],
                    normal: [1.0, 0.0, 0.0],
                    connection_type: ConnectionType::SnapFit,
                    compatible_types: vec![ComponentType::FloorPanel, ComponentType::WallPanel],
                },
            ],
            field_properties: ComponentFieldProperties {
                thermal_conductivity: 0.5,
                acoustic_absorption: 0.3,
                electromagnetic_shielding: 0.7,
                structural_integrity: 0.9,
                field_coupling_strength: 0.2,
            },
            boolean_operations: vec![],
            lod_levels: vec![
                ComponentLodLevel {
                    detail_threshold: 0.0,
                    triangle_count: 200,
                    collision_complexity: CollisionComplexity::Precise,
                    byte_offset: 0,
                    byte_size: 0,
                },
                ComponentLodLevel {
                    detail_threshold: 10.0,
                    triangle_count: 50,
                    collision_complexity: CollisionComplexity::ConvexHull,
                    byte_offset: 0,
                    byte_size: 0,
                },
                ComponentLodLevel {
                    detail_threshold: 50.0,
                    triangle_count: 12,
                    collision_complexity: CollisionComplexity::ConvexHull,
                    byte_offset: 0,
                    byte_size: 0,
                },
            ],
        }
    }

    /// Create a seat component for vehicle interiors
    pub fn seat_template(id: &str, seat_type: &str) -> Self {
        let (width, height, depth) = match seat_type {
            "pilot" => (0.6, 1.2, 0.8),
            "passenger" => (0.55, 1.0, 0.75),
            "cargo" => (0.5, 0.8, 0.7),
            _ => (0.55, 1.0, 0.75),
        };

        ModularComponent {
            id: id.to_string(),
            name: format!("Seat_{}", seat_type),
            component_type: ComponentType::Seat,
            scale: 1.0,
            bounds: BoundingBox {
                min: [-width / 2.0, 0.0, -depth / 2.0],
                max: [width / 2.0, height, depth / 2.0],
            },
            mass_kg: 15.0,
            material_id: "seat_fabric_01".into(),
            connection_points: vec![
                ConnectionPoint {
                    id: "mount_floor".into(),
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, -1.0, 0.0],
                    connection_type: ConnectionType::Bolted,
                    compatible_types: vec![ComponentType::FloorPanel],
                },
            ],
            field_properties: ComponentFieldProperties {
                thermal_conductivity: 0.3,
                acoustic_absorption: 0.6,
                electromagnetic_shielding: 0.2,
                structural_integrity: 0.7,
                field_coupling_strength: 0.1,
            },
            boolean_operations: vec![
                BooleanOperation::SubtractVolume(BoundingBox {
                    min: [-width / 3.0, height * 0.3, -depth / 3.0],
                    max: [width / 3.0, height * 0.7, depth / 3.0],
                }),
            ],
            lod_levels: vec![
                ComponentLodLevel {
                    detail_threshold: 0.0,
                    triangle_count: 1500,
                    collision_complexity: CollisionComplexity::Precise,
                    byte_offset: 0,
                    byte_size: 0,
                },
                ComponentLodLevel {
                    detail_threshold: 5.0,
                    triangle_count: 400,
                    collision_complexity: CollisionComplexity::ConvexHull,
                    byte_offset: 0,
                    byte_size: 0,
                },
                ComponentLodLevel {
                    detail_threshold: 20.0,
                    triangle_count: 100,
                    collision_complexity: CollisionComplexity::ConvexHull,
                    byte_offset: 0,
                    byte_size: 0,
                },
            ],
        }
    }

    /// Apply scale transformation to component
    pub fn apply_scale(&mut self, scale_factor: f32) {
        self.scale *= scale_factor;
        self.bounds.min.iter_mut().for_each(|v| *v *= scale_factor);
        self.bounds.max.iter_mut().for_each(|v| *v *= scale_factor);
        self.mass_kg *= scale_factor.powi(3);
    }

    /// Add a boolean operation to the component
    pub fn add_boolean_operation(&mut self, operation: BooleanOperation) {
        self.boolean_operations.push(operation);
    }

    /// Export component to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl InteriorAssembly {
    /// Create a new empty assembly
    pub fn new(id: &str, name: &str) -> Self {
        InteriorAssembly {
            id: id.to_string(),
            name: name.to_string(),
            components: Vec::new(),
            total_bounds: BoundingBox {
                min: [0.0, 0.0, 0.0],
                max: [0.0, 0.0, 0.0],
            },
            total_mass_kg: 0.0,
            field_signature: FieldSignature {
                thermal_emission: 0.0,
                acoustic_profile: 0.0,
                electromagnetic_signature: 0.0,
                field_distortion: 0.0,
            },
        }
    }

    /// Add a component to the assembly
    pub fn add_component(&mut self, component: AssemblyComponent) {
        self.total_mass_kg += 10.0; // Placeholder mass calculation
        self.components.push(component);
        self.recalculate_bounds();
    }

    /// Recalculate total bounds from all components
    fn recalculate_bounds(&mut self) {
        if self.components.is_empty() {
            return;
        }

        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];

        for comp in &self.components {
            for i in 0..3 {
                min[i] = min[i].min(comp.position[i]);
                max[i] = max[i].max(comp.position[i]);
            }
        }

        self.total_bounds = BoundingBox { min, max };
    }

    /// Calculate field signature from components
    pub fn calculate_field_signature(&mut self) {
        let count = self.components.len() as f32;
        if count > 0.0 {
            self.field_signature = FieldSignature {
                thermal_emission: count * 0.1,
                acoustic_profile: count * 0.05,
                electromagnetic_signature: count * 0.02,
                field_distortion: count * 0.01,
            };
        }
    }

    /// Export assembly to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floor_panel_creation() {
        let panel = ModularComponent::floor_panel_template("fp001", 2.0, 3.0);
        assert_eq!(panel.component_type, ComponentType::FloorPanel);
        assert_eq!(panel.connection_points.len(), 4);
    }

    #[test]
    fn test_seat_creation() {
        let seat = ModularComponent::seat_template("s001", "pilot");
        assert_eq!(seat.component_type, ComponentType::Seat);
        assert!(!seat.boolean_operations.is_empty());
    }

    #[test]
    fn test_component_scaling() {
        let mut panel = ModularComponent::floor_panel_template("fp001", 2.0, 3.0);
        let original_mass = panel.mass_kg;
        panel.apply_scale(2.0);
        assert!((panel.scale - 2.0).abs() < 0.001);
        assert!((panel.mass_kg - original_mass * 8.0).abs() < 0.001);
    }

    #[test]
    fn test_assembly_creation() {
        let mut assembly = InteriorAssembly::new("asm001", "TestInterior");
        assert_eq!(assembly.components.len(), 0);
        
        let comp = AssemblyComponent {
            component_id: "fp001".into(),
            instance_id: "inst001".into(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: 1.0,
            parent_connection: None,
            child_connection: None,
        };
        assembly.add_component(comp);
        assert_eq!(assembly.components.len(), 1);
    }

    #[test]
    fn test_component_serialization() {
        let panel = ModularComponent::floor_panel_template("fp001", 2.0, 3.0);
        let json = panel.to_json().expect("Failed to serialize component");
        assert!(json.contains("FloorPanel"));
    }
}
