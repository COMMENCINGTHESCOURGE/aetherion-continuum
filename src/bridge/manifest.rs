// ═══════════════════════════════════════════════════════════════
// AETHERION-CONTINUUM — Bridge: Export Manifest
// Nanite/Lumen mesh export, Blender Geometry Nodes bridge,
// UE5 Field Asset pipeline descriptors.
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportManifest {
    pub engine: String,
    pub version: String,
    pub asset_type: AssetType,
    pub bounds: BoundingBox,
    pub lod_levels: Vec<LodLevel>,
    pub material_mapping: MaterialMapping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AssetType {
    MeshletCluster, // Nanite-compatible meshlet export
    HeightField,    // Blender Geometry Nodes heightmap
    FieldTensor,    // Raw 6D field data for UE5 Field Asset
    SparseOctree,   // Octree structure for external engines
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LodLevel {
    pub detail: f32,
    pub meshlet_count: u32,
    pub triangle_count: u32,
    pub byte_offset: u64,
    pub byte_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialMapping {
    pub field_channels: Vec<FieldChannelMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldChannelMapping {
    pub channel_name: String,
    pub target_material_param: String,
    pub range: [f32; 2],
}

impl ExportManifest {
    pub fn new_nanite_meshlet() -> Self {
        ExportManifest {
            engine: "Unreal Engine 5".into(),
            version: "5.4+".into(),
            asset_type: AssetType::MeshletCluster,
            bounds: BoundingBox {
                min: [0.0; 3],
                max: [16384.0; 3],
            },
            lod_levels: vec![
                LodLevel {
                    detail: 0.0,
                    meshlet_count: 128,
                    triangle_count: 8192,
                    byte_offset: 0,
                    byte_size: 0,
                },
                LodLevel {
                    detail: 0.5,
                    meshlet_count: 2048,
                    triangle_count: 131072,
                    byte_offset: 0,
                    byte_size: 0,
                },
                LodLevel {
                    detail: 1.0,
                    meshlet_count: 32768,
                    triangle_count: 2097152,
                    byte_offset: 0,
                    byte_size: 0,
                },
            ],
            material_mapping: MaterialMapping {
                field_channels: vec![
                    FieldChannelMapping {
                        channel_name: "density".into(),
                        target_material_param: "BaseColor".into(),
                        range: [0.0, 1.0],
                    },
                    FieldChannelMapping {
                        channel_name: "cohesion".into(),
                        target_material_param: "Roughness".into(),
                        range: [0.0, 1.0],
                    },
                    FieldChannelMapping {
                        channel_name: "moisture".into(),
                        target_material_param: "Specular".into(),
                        range: [0.0, 1.0],
                    },
                ],
            },
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serialization() {
        let manifest = ExportManifest::new_nanite_meshlet();
        let json = manifest.to_json().expect("Failed to serialize manifest");
        assert!(json.contains("Unreal Engine 5"));
        assert!(json.contains("MeshletCluster"));
    }

    #[test]
    fn test_manifest_deserialization() {
        let json_data = r#"{
            "engine": "Custom Engine",
            "version": "1.0",
            "asset_type": "FieldTensor",
            "bounds": {
                "min": [0.0, 0.0, 0.0],
                "max": [10.0, 10.0, 10.0]
            },
            "lod_levels": [],
            "material_mapping": {
                "field_channels": []
            }
        }"#;

        let manifest: ExportManifest =
            serde_json::from_str(json_data).expect("Failed to deserialize manifest");
        assert_eq!(manifest.engine, "Custom Engine");
        assert!(matches!(manifest.asset_type, AssetType::FieldTensor));
    }
}
