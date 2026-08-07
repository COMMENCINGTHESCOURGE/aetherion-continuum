// ═══════════════════════════════════════════════════════════════
// AETHERION-CONTINUUM — Bridge: OpenCode Modular Interiors
// Modular interior system for vehicles and structures with field-aware
// component assembly and boolean operations
// 
// PHASE 2 UPDATE: Field-Aware Component System (Islamic Golden Age)
// Democratizes field physics across all components universally
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use crate::field::{Tensor6, Archetype};

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

/// PHASE 2: Field-Aware Trait - Universal interface for field-reactive components
/// Democratizes field physics access across all modules (Islamic Golden Age principle)
pub trait FieldAware {
    /// Query ambient field tensor at component position
    fn query_field(&self, field_tensor: &Tensor6) -> FieldInteraction;
    
    /// React to field changes by modifying component state
    fn react_to_field(&mut self, interaction: &FieldInteraction);
    
    /// Get component's contribution to global field
    fn get_field_contribution(&self) -> Tensor6;
    
    /// Map component archetype to ECS system
    fn archetype(&self) -> Archetype;
}

/// Field interaction result from component-field coupling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInteraction {
    pub force_vector: [f32; 3],
    pub torque_vector: [f32; 3],
    pub energy_transfer: f32,
    pub field_distortion: f32,
    pub coupling_coefficient: f32,
}

impl FieldInteraction {
    pub fn zero() -> Self {
        Self {
            force_vector: [0.0; 3],
            torque_vector: [0.0; 3],
            energy_transfer: 0.0,
            field_distortion: 0.0,
            coupling_coefficient: 0.0,
        }
    }
}

/// PHASE 2: Central registry of universal field parameters
/// Prevents localized math assumptions - all components reference identical constants
#[derive(Debug, Clone)]
pub struct FieldParameterRegistry {
    pub gravitational_constant: f32,
    pub permittivity_free_space: f32,
    pub permeability_free_space: f32,
    pub speed_of_light: f32,
    pub planck_constant: f32,
    pub boltzmann_constant: f32,
    pub reference_temperature: f32,
    pub reference_density: f32,
}

impl Default for FieldParameterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldParameterRegistry {
    pub fn new() -> Self {
        Self {
            gravitational_constant: 6.674e-11,
            permittivity_free_space: 8.854e-12,
            permeability_free_space: 1.257e-6,
            speed_of_light: 2.998e8,
            planck_constant: 6.626e-34,
            boltzmann_constant: 1.381e-23,
            reference_temperature: 293.15, // 20°C in Kelvin
            reference_density: 1.225,      // Air density at sea level kg/m³
        }
    }
    
    /// Global singleton accessor - ensures all components use identical constants
    pub fn global() -> &'static Self {
        static REGISTRY: FieldParameterRegistry = FieldParameterRegistry {
            gravitational_constant: 6.674e-11,
            permittivity_free_space: 8.854e-12,
            permeability_free_space: 1.257e-6,
            speed_of_light: 2.998e8,
            planck_constant: 6.626e-34,
            boltzmann_constant: 1.381e-23,
            reference_temperature: 293.15,
            reference_density: 1.225,
        };
        &REGISTRY
    }
}

/// PHASE 2: Ambient field emitter - projects dynamic fields onto nearby FieldAware components
/// Eliminates hardcoded coupling; components react based on proximity and field strength
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientFieldEmitter {
    pub id: String,
    pub position: [f32; 3],
    pub emission_radius: f32,
    pub field_type: FieldType,
    pub intensity: f32,
    pub frequency_hz: f32,
    pub modulation: FieldModulation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Gravitational,
    Electromagnetic,
    Thermal,
    Acoustic,
    Quantum,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldModulation {
    Constant,
    Sinusoidal { amplitude: f32, phase: f32 },
    Pulsed { duty_cycle: f32, period_ms: f32 },
    Random { variance: f32 },
}

impl AmbientFieldEmitter {
    pub fn new(id: &str, position: [f32; 3], field_type: FieldType, intensity: f32) -> Self {
        Self {
            id: id.to_string(),
            position,
            emission_radius: 10.0,
            field_type,
            intensity,
            frequency_hz: 0.0,
            modulation: FieldModulation::Constant,
        }
    }
    
    /// Calculate field strength at a given point with inverse-square law
    pub fn field_strength_at(&self, point: [f32; 3]) -> f32 {
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        let dz = point[2] - self.position[2];
        let distance_squared = dx * dx + dy * dy + dz * dz;
        
        if distance_squared < 0.01 {
            return self.intensity; // Cap at close range
        }
        
        if distance_squared > self.emission_radius * self.emission_radius {
            return 0.0;
        }
        
        // Inverse-square law with smooth falloff
        self.intensity / (distance_squared + 0.1)
    }
    
    /// Project field tensor onto a FieldAware component
    pub fn project_onto(&self, component: &dyn FieldAware) -> FieldInteraction {
        let strength = self.field_strength_at(component.query_field(&Tensor6::ZERO).force_vector);
        
        if strength < 1e-6 {
            return FieldInteraction::zero();
        }
        
        let direction = [
            component.query_field(&Tensor6::ZERO).force_vector[0] - self.position[0],
            component.query_field(&Tensor6::ZERO).force_vector[1] - self.position[1],
            component.query_field(&Tensor6::ZERO).force_vector[2] - self.position[2],
        ];
        
        let distance = (direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2)).sqrt();
        let normalized_direction = if distance > 0.0 {
            [direction[0] / distance, direction[1] / distance, direction[2] / distance]
        } else {
            [0.0, 1.0, 0.0]
        };
        
        FieldInteraction {
            force_vector: [
                normalized_direction[0] * strength,
                normalized_direction[1] * strength,
                normalized_direction[2] * strength,
            ],
            torque_vector: [0.0; 3], // Simplified - would need component orientation
            energy_transfer: strength * 0.1,
            field_distortion: strength * 0.05,
            coupling_coefficient: strength,
        }
    }
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

    // PHASE 2 TESTS: Field-Aware Component System
    
    #[test]
    fn test_field_parameter_registry_global() {
        let registry = FieldParameterRegistry::global();
        assert!((registry.gravitational_constant - 6.674e-11).abs() < 1e-15);
        assert!((registry.speed_of_light - 2.998e8).abs() < 1e3);
        assert!((registry.reference_temperature - 293.15).abs() < 0.01);
    }

    #[test]
    fn test_ambient_field_emitter_creation() {
        let emitter = AmbientFieldEmitter::new(
            "emitter001",
            [0.0, 0.0, 0.0],
            FieldType::Electromagnetic,
            100.0,
        );
        assert_eq!(emitter.id, "emitter001");
        assert_eq!(emitter.intensity, 100.0);
        assert_eq!(emitter.emission_radius, 10.0);
    }

    #[test]
    fn test_field_strength_inverse_square_law() {
        let emitter = AmbientFieldEmitter::new(
            "emitter001",
            [0.0, 0.0, 0.0],
            FieldType::Gravitational,
            100.0,
        );
        
        // At origin (very close), should be capped at intensity
        let strength_at_center = emitter.field_strength_at([0.0, 0.0, 0.0]);
        assert!((strength_at_center - 100.0).abs() < 0.01);
        
        // At distance 1.0, should follow inverse-square law
        let strength_at_1m = emitter.field_strength_at([1.0, 0.0, 0.0]);
        let expected = 100.0 / (1.0 + 0.1);
        assert!((strength_at_1m - expected).abs() < 0.01);
        
        // At distance 5.0
        let strength_at_5m = emitter.field_strength_at([5.0, 0.0, 0.0]);
        let expected_5m = 100.0 / (25.0 + 0.1);
        assert!((strength_at_5m - expected_5m).abs() < 0.01);
        
        // Beyond emission radius (10.0), should be zero
        let strength_beyond = emitter.field_strength_at([15.0, 0.0, 0.0]);
        assert!(strength_beyond < 1e-6);
    }

    #[test]
    fn test_field_interaction_zero() {
        let interaction = FieldInteraction::zero();
        assert_eq!(interaction.force_vector, [0.0; 3]);
        assert_eq!(interaction.torque_vector, [0.0; 3]);
        assert!((interaction.energy_transfer - 0.0).abs() < 0.001);
        assert!((interaction.field_distortion - 0.0).abs() < 0.001);
        assert!((interaction.coupling_coefficient - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_field_modulation_types() {
        let constant = FieldModulation::Constant;
        let sinusoidal = FieldModulation::Sinusoidal { amplitude: 0.5, phase: 1.57 };
        let pulsed = FieldModulation::Pulsed { duty_cycle: 0.3, period_ms: 100.0 };
        let random = FieldModulation::Random { variance: 0.1 };
        
        // Just verify they can be constructed and differentiated
        match constant {
            FieldModulation::Constant => {},
            _ => panic!("Expected Constant"),
        }
        match sinusoidal {
            FieldModulation::Sinusoidal { amplitude, phase } => {
                assert!((amplitude - 0.5).abs() < 0.001);
                assert!((phase - 1.57).abs() < 0.001);
            },
            _ => panic!("Expected Sinusoidal"),
        }
    }
}
