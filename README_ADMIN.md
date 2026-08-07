# Aetherion Continuum - Administrative Field System

**Version:** 0.2.0  
**License:** MIT  
**Status:** Alpha - Jurisdictional Foundation

A **Field-Native, Conservation-Enforced, Planet-Scale Simulation Engine** with full administrative hierarchy support (County → City → Borough → Neighborhood → Parcel).

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Godot 4 Frontend                         │
│  (Rendering, UI, User Interaction, Vehicle Control)         │
└─────────────────────┬───────────────────────────────────────┘
                      │ GDExtension FFI
┌─────────────────────▼───────────────────────────────────────┐
│              Aetherion Bridge (Rust)                        │
│  (AdminTree API, Rule Resolution, Resource Aggregation)     │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│           Aetherion Core + Parser                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ AdminTree    │  │ OSM Parser   │  │ Rule Solver      │   │
│  │ Hierarchy    │  │ PBF→JSON     │  │ Priority Override│   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│              Data Layer                                     │
│  OSM .pbf files │ GMNS networks │ JSON exports             │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Workspace Structure

```
aetherion-continuum/
├── Cargo.toml              # Root workspace config
├── core/                   # Administrative hierarchy engine
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # AdminTree, AdminLevel, RuleStack
├── parser/                 # OSM boundary parser
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # BoundaryParser
│       └── main.rs         # CLI: parse_boundaries
├── bridge/                 # GDExtension for Godot 4
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # AetherionBridge API
├── godot/                  # Godot 4 project
│   ├── project.godot
│   ├── addons/
│   │   └── aetherion_bridge/
│   ├── scripts/            # GDScript integration
│   └── scenes/
└── data/                   # Sample data & exports
    └── sample_admin_tree.json
```

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Godot 4.2+ (https://godotengine.org/download)
```

### Build the Rust Libraries

```bash
cd /workspace

# Build all workspace members
cargo build --release

# Build with GDExtension support (for Godot integration)
cargo build --release --features godot-extension

# Run tests
cargo test
```

### Parse OSM Data

```bash
# Download OSM data (example: King County, WA)
wget https://download.geofabrik.de/northamerica/us/washington-latest.osm.pbf

# Parse administrative boundaries
cargo run --bin parse_boundaries -- washington-latest.osm.pbf admin_tree.json
```

### Integrate with Godot

1. Copy the built library to Godot addons folder:
   ```bash
   cp target/release/libaetherion_bridge.so godot/addons/aetherion_bridge/
   # On Windows: copy target\release\aetherion_bridge.dll
   # On macOS: cp target/release/libaetherion_bridge.dylib
   ```

2. Open `godot/project.godot` in Godot 4.2+

3. Enable the AetherionBridge plugin in Project Settings → Plugins

## 📖 API Reference

### Rust Core API

```rust
use aetherion_core::{AdminTree, AdministrativeNode, AdminLevel};

// Create admin tree
let mut tree = AdminTree::new();

// Find node at coordinates (lat, lon)
let point = Point::new(lon, lat);
if let Some(node) = tree.find_node_at_point(&point) {
    println!("Location: {} ({:?})", node.name, node.level);
}

// Resolve rules (child overrides parent)
let rules = tree.resolve_rules(&point);
if let Some(speed_limit) = rules.get("speed_limit") {
    println!("Speed limit: {}", speed_limit.value);
}

// Aggregate resources up the hierarchy
tree.aggregate_resources();
```

### GDScript Integration (via GDExtension)

```gdscript
# In your Godot script
var bridge = AetherionBridge.new()

# Load admin tree from JSON
bridge.load_from_json("res://data/admin_tree.json")

# Find which neighborhood contains a coordinate
var node_id = bridge.find_node_at_coords(47.6062, -122.3321)
print("Located in: ", node_id)

# Get resolved rules at location
var rules_json = bridge.get_rules_at_coords(47.6062, -122.3321)
var rules = JSON.parse_string(rules_json)
print("Speed limit: ", rules.get("speed_limit"))

# Update resources (e.g., from building simulation)
bridge.update_resources("neighborhood_123", 5000, 250000.0, 0.85)

# Aggregate up the hierarchy
bridge.aggregate_resources()

# Get community sentiment
var profile_json = bridge.get_community_profile("borough_capitol_hill")
var profile = JSON.parse_string(profile_json)
print("Happiness: ", profile.happiness)
```

## 🔧 Key Features

### 1. Administrative Hierarchy
- **County** (admin_level=6): Regional infrastructure, major roads
- **City** (admin_level=8): Municipal services, zoning laws
- **Borough** (admin_level=9): Urban character, local ordinances
- **Neighborhood** (place=neighbourhood): Social dynamics, HOA rules
- **Parcel** (landuse/addr:*): Private property, driveways, garages

### 2. Rule Propagation System
Rules flow **downward** with priority-based override:
- Lower levels (Parcel) override higher levels (County)
- Time-of-day restrictions supported
- Example: County speed limit 55mph → Neighborhood overrides to 25mph

### 3. Resource Aggregation
Resources flow **upward** from children to parents:
- Population, tax revenue, utility demand
- Noise index, traffic volume
- Automatic aggregation via `tree.aggregate_resources()`

### 4. Community Dynamics
Each node has social profiles affecting simulation:
- Economic status (Wealthy/Industrial/Subsistence)
- Political leaning (Pro-Development/NIMBY/Balanced)
- Happiness metric (0.0-1.0)
- Density preference

### 5. Aesthetic Profiles
Procedural generation modifiers:
- Primary materials (Brick/Glass/Concrete/Wood)
- Roof styles (Flat/Pitched/Terraced/Domed)
- Greenery density (0.0-1.0)
- Color palettes for buildings

## 🗺️ OSM Tagging Strategy

| Feature | OSM Tags | Admin Level |
|---------|----------|-------------|
| County | `boundary=administrative` + `admin_level=6` | County |
| City | `boundary=administrative` + `admin_level=8` | City |
| Borough | `boundary=administrative` + `admin_level=9` | Borough |
| Neighborhood | `place=neighbourhood` or `boundary=locality` | Neighborhood |
| Driveway | `highway=service` + `service=driveway` | Governed by Parcel |
| Garage | `building=garage` | Governed by Parcel |

## 🎮 Integration with Vehicle/City Game

This engine provides the **jurisdictional brain** for your city builder:

1. **Driveway/Garage Rules**: HOA covenants, setback requirements, material restrictions
2. **Traffic Simulation**: Speed limits, vehicle type restrictions, parking duration
3. **Building Growth**: Zoning laws, height limits, density preferences
4. **Citizen Feedback**: Happiness affects political leaning, which affects policy changes

## 📝 Next Steps

### Phase 1: Core Infrastructure ✅
- [x] AdminTree data structures
- [x] Rule priority system
- [x] Resource aggregation
- [x] OSM parser skeleton

### Phase 2: GMNS Integration (Next)
- [ ] Extend GMNS Link/Node with `admin_id` field
- [ ] Road network assignment to jurisdictions
- [ ] Driveway/garage extensions to GMNS schema

### Phase 3: Godot Integration
- [ ] Complete GDExtension bindings
- [ ] Debug overlay for admin boundaries
- [ ] Real-time rule visualization

### Phase 4: Simulation Features
- [ ] Vehicle agent behavior with rule compliance
- [ ] Building growth based on zoning + community profile
- [ ] Dynamic policy changes based on happiness

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Run `cargo test` before submitting PR
4. Document new features in README

## 📄 License

MIT License - See LICENSE file for details.

---

**Built for the COMMENCINGTHESCOURGE organization**  
*Field-Native • Conservation-Enforced • Jurisdiction-Aware*
