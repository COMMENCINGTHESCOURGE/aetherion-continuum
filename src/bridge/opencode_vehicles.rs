// ═══════════════════════════════════════════════════════════════
// AETHERION-CONTINUUM — Bridge: OpenCode Vehicles
// Vehicle definitions for simulation entities with field interactions
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

/// OpenCode Vehicle types for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VehicleType {
    Ground,
    Aerial,
    Maritime,
    Subsurface,
    Orbital,
}

/// Vehicle propulsion system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropulsionSystem {
    pub thrust_vector: [f32; 3],
    pub max_thrust: f32,
    pub efficiency: f32,
    pub fuel_type: String,
}

/// Vehicle sensor suite for field interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSuite {
    pub lidar_resolution: f32,
    pub radar_range: f32,
    pub field_sensitivity: f32,
    pub sampling_rate_hz: u32,
}

/// OpenCode Vehicle definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeVehicle {
    pub id: String,
    pub name: String,
    pub vehicle_type: VehicleType,
    pub scale: f32,
    pub bounds: BoundingBox,
    pub mass_kg: f32,
    pub propulsion: Option<PropulsionSystem>,
    pub sensors: Option<SensorSuite>,
    pub field_interaction_coefficients: FieldInteractionCoeffs,
    pub lod_levels: Vec<VehicleLodLevel>,
}

/// Field interaction coefficients for vehicle-environment coupling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInteractionCoeffs {
    pub drag_coefficient: f32,
    pub lift_coefficient: f32,
    pub thermal_signature: f32,
    pub acoustic_signature: f32,
    pub electromagnetic_cross_section: f32,
}

/// Level of Detail for vehicle rendering/simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLodLevel {
    pub detail_threshold: f32,
    pub triangle_count: u32,
    pub physics_fidelity: PhysicsFidelity,
    pub byte_offset: u64,
    pub byte_size: u64,
}

/// Physics fidelity levels for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsFidelity {
    Kinematic,      // Position-only, no forces
    DynamicSimple,  // Basic Newtonian physics
    DynamicFull,    // Full rigid body dynamics
    CFDCoupled,     // Computational fluid dynamics coupled
}

/// Bounding box for vehicle geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl OpenCodeVehicle {
    /// Create a standard ground vehicle template
    pub fn ground_vehicle_template(id: &str, name: &str) -> Self {
        OpenCodeVehicle {
            id: id.to_string(),
            name: name.to_string(),
            vehicle_type: VehicleType::Ground,
            scale: 1.0,
            bounds: BoundingBox {
                min: [-2.5, 0.0, -1.0],
                max: [2.5, 1.8, 5.0],
            },
            mass_kg: 2000.0,
            propulsion: Some(PropulsionSystem {
                thrust_vector: [0.0, 0.0, 1.0],
                max_thrust: 5000.0,
                efficiency: 0.85,
                fuel_type: "electric".into(),
            }),
            sensors: Some(SensorSuite {
                lidar_resolution: 0.05,
                radar_range: 200.0,
                field_sensitivity: 0.001,
                sampling_rate_hz: 60,
            }),
            field_interaction_coefficients: FieldInteractionCoeffs {
                drag_coefficient: 0.32,
                lift_coefficient: 0.0,
                thermal_signature: 0.5,
                acoustic_signature: 0.3,
                electromagnetic_cross_section: 1.2,
            },
            lod_levels: vec![
                VehicleLodLevel {
                    detail_threshold: 0.0,
                    triangle_count: 50000,
                    physics_fidelity: PhysicsFidelity::DynamicFull,
                    byte_offset: 0,
                    byte_size: 0,
                },
                VehicleLodLevel {
                    detail_threshold: 50.0,
                    triangle_count: 10000,
                    physics_fidelity: PhysicsFidelity::DynamicSimple,
                    byte_offset: 0,
                    byte_size: 0,
                },
                VehicleLodLevel {
                    detail_threshold: 200.0,
                    triangle_count: 2000,
                    physics_fidelity: PhysicsFidelity::Kinematic,
                    byte_offset: 0,
                    byte_size: 0,
                },
            ],
        }
    }

    /// Create an aerial vehicle template (drone/UAV)
    pub fn aerial_drone_template(id: &str, name: &str) -> Self {
        OpenCodeVehicle {
            id: id.to_string(),
            name: name.to_string(),
            vehicle_type: VehicleType::Aerial,
            scale: 0.1,
            bounds: BoundingBox {
                min: [-0.5, -0.2, -0.5],
                max: [0.5, 0.3, 0.5],
            },
            mass_kg: 2.5,
            propulsion: Some(PropulsionSystem {
                thrust_vector: [0.0, 1.0, 0.0],
                max_thrust: 50.0,
                efficiency: 0.75,
                fuel_type: "electric".into(),
            }),
            sensors: Some(SensorSuite {
                lidar_resolution: 0.02,
                radar_range: 50.0,
                field_sensitivity: 0.0001,
                sampling_rate_hz: 120,
            }),
            field_interaction_coefficients: FieldInteractionCoeffs {
                drag_coefficient: 0.15,
                lift_coefficient: 0.8,
                thermal_signature: 0.2,
                acoustic_signature: 0.4,
                electromagnetic_cross_section: 0.3,
            },
            lod_levels: vec![
                VehicleLodLevel {
                    detail_threshold: 0.0,
                    triangle_count: 8000,
                    physics_fidelity: PhysicsFidelity::CFDCoupled,
                    byte_offset: 0,
                    byte_size: 0,
                },
                VehicleLodLevel {
                    detail_threshold: 20.0,
                    triangle_count: 2000,
                    physics_fidelity: PhysicsFidelity::DynamicFull,
                    byte_offset: 0,
                    byte_size: 0,
                },
                VehicleLodLevel {
                    detail_threshold: 100.0,
                    triangle_count: 500,
                    physics_fidelity: PhysicsFidelity::Kinematic,
                    byte_offset: 0,
                    byte_size: 0,
                },
            ],
        }
    }

    /// Apply scale transformation to vehicle
    pub fn apply_scale(&mut self, scale_factor: f32) {
        self.scale *= scale_factor;
        self.bounds.min.iter_mut().for_each(|v| *v *= scale_factor);
        self.bounds.max.iter_mut().for_each(|v| *v *= scale_factor);
        self.mass_kg *= scale_factor.powi(3); // Mass scales with volume
    }

    /// Export vehicle to JSON manifest
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_vehicle_creation() {
        let vehicle = OpenCodeVehicle::ground_vehicle_template("v001", "TestCar");
        assert_eq!(vehicle.vehicle_type, VehicleType::Ground);
        assert!(vehicle.propulsion.is_some());
        assert!(vehicle.sensors.is_some());
    }

    #[test]
    fn test_aerial_drone_creation() {
        let vehicle = OpenCodeVehicle::aerial_drone_template("d001", "TestDrone");
        assert_eq!(vehicle.vehicle_type, VehicleType::Aerial);
        assert!((vehicle.scale - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_vehicle_scaling() {
        let mut vehicle = OpenCodeVehicle::ground_vehicle_template("v001", "TestCar");
        let original_mass = vehicle.mass_kg;
        vehicle.apply_scale(2.0);
        assert!((vehicle.scale - 2.0).abs() < 0.001);
        assert!((vehicle.mass_kg - original_mass * 8.0).abs() < 0.001); // 2^3 = 8
    }

    #[test]
    fn test_vehicle_serialization() {
        let vehicle = OpenCodeVehicle::ground_vehicle_template("v001", "TestCar");
        let json = vehicle.to_json().expect("Failed to serialize vehicle");
        assert!(json.contains("TestCar"));
        assert!(json.contains("Ground"));
    }
}
