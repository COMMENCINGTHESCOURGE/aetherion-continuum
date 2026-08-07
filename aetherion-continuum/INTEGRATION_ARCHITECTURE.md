# Aetherion Continuum - Integration Architecture Document

## Overview

This document describes how to integrate the **Aetherion Continuum** Rust backend with **Godot 4** frontend and **OpenStreetMap/GMNS** data sources to create a unified hyperrealistic simulation platform.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        GODOT 4 FRONTEND                         │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ 3D Renderer │  │ Vehicle UI   │  │ City Builder Tools  │    │
│  │ (Vulkan)    │  │ (Modular)    │  │ (Zoning, Roads)     │    │
│  └──────┬──────┘  └──────┬───────┘  └──────────┬──────────┘    │
│         │                │                      │               │
│         └────────────────┼──────────────────────┘               │
│                          │                                      │
│              ┌───────────▼───────────┐                          │
│              │   GDExtension Layer   │                          │
│              │   (godot-rust FFI)    │                          │
│              └───────────┬───────────┘                          │
└──────────────────────────┼──────────────────────────────────────┘
                           │
                    RPC / Shared Memory
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                    AETHERION CONTINUUM (Rust)                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              Administrative Field System                │    │
│  │  County → City → Borough → Neighborhood → Parcel        │    │
│  │  • Rule Propagation (Downward)                          │    │
│  │  • Resource Aggregation (Upward)                        │    │
│  │  • Point-in-Polygon Queries                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────┐  ┌──────────────────┐  ┌──────────────┐    │
│  │ OSM Parser      │  │ GMNS Bridge      │  │ Field Tensor │    │
│  │ (.pbf → Tree)   │  │ (Links/Nodes)    │  │ (8D WGSL)    │    │
│  └─────────────────┘  └──────────────────┘  └──────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │           Conservation Enforcement Engine               │    │
│  │  Mass/Energy/Momentum conservation with error < 1e-5    │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                    File I/O / API
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                      DATA SOURCES                               │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐    │
│  │ OpenStreetMap│  │ GMNS Files   │  │ Symbios-Tensor      │    │
│  │ (.osm.pbf)   │  │ (CSV/JSON)   │  │ (Procedural Mesh)   │    │
│  └─────────────┘  └──────────────┘  └─────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Module 1: Administrative Hierarchy Integration

### Data Flow: OSM → AdminTree → Godot

1. **OSM Boundary Extraction** (Rust)
   ```rust
   use aetherion_continuum::osm_parser;
   
   let tree = osm_parser::parse_admin_boundaries("seattle_area.osm.pbf")?;
   // Returns AdminTree with County/City/Borough/Neighborhood nodes
   ```

2. **Rule Resolution for Roads/Driveways** (Rust)
   ```rust
   use geo::Point;
   
   // A driveway at this coordinate
   let driveway_location = Point::new(-122.3321, 47.6062);
   
   // Find governing administrative node
   let node = tree.find_deepest_node(&driveway_location).unwrap();
   
   // Get all applicable rules (HOA + City + County)
   let rules = tree.get_effective_rules(&node.id);
   
   // Check if motorcycle parking is allowed
   if let Some(vehicle_rule) = rules.restrictions.get("vehicle_type") {
       println!("Driveway restriction: {}", vehicle_rule.value);
   }
   ```

3. **Visualization in Godot** (GDScript via GDExtension)
   ```gdscript
   # Draw administrative boundaries with color coding
   func draw_admin_boundaries(admin_tree: AdminTreeRef):
       for node_id in admin_tree.get_node_ids():
           var node = admin_tree.get_node(node_id)
           var color = _get_color_for_level(node.level)
           
           # Draw polygon boundary
           var polygon = node.boundary
           _draw_polygon(polygon, color)
           
           # Display rules on hover
           var rules = admin_tree.get_effective_rules(node_id)
           _create_tooltip(node.name, rules)
   ```

---

## Module 2: GMNS Network + Administrative Rules

### Linking Physical Infrastructure to Governance

```rust
use aetherion_continuum::{AdminTree, gmns_bridge};
use gmns::{Network, Link};

// Load GMNS network (roads, driveways, bike lanes)
let network = gmns_bridge::load_from_csv("network_links.csv")?;

// Assign each link to its administrative zone
for link in &mut network.links {
    let centroid = link.geometry.centroid();
    
    if let Some(admin_node) = admin_tree.find_deepest_node(&centroid) {
        link.admin_id = Some(admin_node.id.clone());
        link.effective_rules = admin_tree.get_effective_rules(&admin_node.id);
    }
}

// Example: A driveway link inherits neighborhood HOA rules
if link.link_type == "driveway" {
    println!("Driveway {} governed by:", link.id);
    for (key, rule) in &link.effective_rules.restrictions {
        println!("  {}: {} (priority {})", key, rule.value, rule.priority);
    }
}
```

### GMNS Extension for Private Infrastructure

| Field | Type | Description |
|-------|------|-------------|
| `link_type` | enum | `highway`, `street`, `driveway`, `garage_aisle` |
| `access_level` | enum | `public`, `private`, `gated_community` |
| `admin_id` | string | Reference to AdministrativeNode |
| `hoa_restrictions` | JSON | HOA-specific rules (parking, aesthetics) |
| `gateway_node` | bool | Transition point between public/private |

---

## Module 3: Vehicle System Integration

### Modular Vehicles + Jurisdictional Rules

```rust
// Vehicle definition from your original Godot design
struct Vehicle {
    parts: Vec<VehiclePart>,  // Chassis, Wheels, Engine
    total_mass: f32,
    vehicle_type: String,     // "sedan", "truck", "motorcycle"
}

// When spawning a vehicle in Godot
fn spawn_vehicle_in_garage(vehicle: &Vehicle, garage_location: Point<f64>) {
    // 1. Find the administrative zone
    let admin_node = admin_tree.find_deepest_node(&garage_location).unwrap();
    
    // 2. Get effective rules
    let rules = admin_tree.get_effective_rules(&admin_node.id);
    
    // 3. Validate vehicle against local restrictions
    if let Some(vehicle_rule) = rules.restrictions.get("vehicle_type") {
        if !vehicle_allowed(&vehicle.vehicle_type, &vehicle_rule.value) {
            println!("Vehicle type {} not allowed in this zone!", vehicle.vehicle_type);
            return;
        }
    }
    
    // 4. Check driveway length vs lot size
    if let Some(lot_size_rule) = rules.restrictions.get("min_lot_size") {
        let required_length = calculate_driveway_length(&vehicle);
        if required_length > lot_size_rule.value.parse::<f32>().unwrap() {
            println!("Driveway too short for this vehicle!");
        }
    }
    
    // 5. Spawn in Godot
    godot_spawn_vehicle(vehicle, garage_location);
}
```

---

## Module 4: Building Growth + Zoning Laws

### Administrative Influence on City Simulation

```rust
// Building growth logic from your original design
fn try_grow_building(zone: &Zone, admin_tree: &AdminTree) -> Option<Building> {
    let centroid = zone.get_centroid();
    let admin_node = admin_tree.find_deepest_node(&centroid)?;
    let rules = admin_tree.get_effective_rules(&admin_node.id);
    
    // Check zoning compatibility
    if let Some(zoning_rule) = rules.restrictions.get("allowed_uses") {
        if !zoning_allows(zone.zone_type, &zoning_rule.value) {
            return None; // Zone doesn't allow this building type
        }
    }
    
    // Check density limits from city/borough level
    if let Some(density_rule) = rules.restrictions.get("max_density") {
        if zone.current_density >= density_rule.value.parse::<f32>().unwrap() {
            return None; // Maximum density reached
        }
    }
    
    // Check community profile (NIMBY vs Pro-Development)
    let community = &admin_node.community_profile;
    if community.political_leaning == "NIMBY" && community.happiness_index < 0.3 {
        // Residents oppose new construction
        return None;
    }
    
    // All checks passed - grow building
    Some(spawn_building(zone, admin_node.aesthetic_profile.clone()))
}
```

---

## Module 5: Godot GDExtension API Design

### Rust Side (GDExtension Bindings)

```rust
use godot::prelude::*;
use aetherion_continuum::{AdminTree, AdministrativeNode};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct AdminTreeWrapper {
    tree: AdminTree,
    base: Base<Node>,
}

#[godot_api]
impl AdminTreeWrapper {
    #[func]
    pub fn load_from_osm(&mut self, pbf_path: String) -> bool {
        match osm_parser::parse_admin_boundaries(&pbf_path) {
            Ok(tree) => {
                self.tree = tree;
                true
            }
            Err(_) => false,
        }
    }
    
    #[func]
    pub fn get_zone_at_position(&self, x: f64, z: f64) -> Variant {
        let point = Point::new(x, z);
        if let Some(node) = self.tree.find_deepest_node(&point) {
            // Return as Godot Dictionary
            let mut dict = Dictionary::new();
            dict.set("id".to_variant(), node.id.to_variant());
            dict.set("name".to_variant(), node.name.to_variant());
            dict.set("level".to_variant(), format!("{:?}", node.level).to_variant());
            dict.set("happiness".to_variant(), node.community_profile.happiness_index.to_variant());
            dict.to_variant()
        } else {
            Variant::nil()
        }
    }
    
    #[func]
    pub fn get_effective_rules(&self, node_id: String) -> Dictionary {
        let rules = self.tree.get_effective_rules(&node_id);
        let mut dict = Dictionary::new();
        
        for (key, restriction) in &rules.restrictions {
            let mut rule_dict = Dictionary::new();
            rule_dict.set("value".to_variant(), restriction.value.to_variant());
            rule_dict.set("priority".to_variant(), restriction.priority.to_variant());
            dict.set(key.to_variant(), rule_dict.to_variant());
        }
        
        dict
    }
    
    #[func]
    pub fn draw_boundaries(&self, canvas: &mut CanvasLayer) {
        for node in self.tree.nodes.values() {
            let color = match node.level {
                AdminLevel::County => Color::RED,
                AdminLevel::City => Color::GREEN,
                AdminLevel::Borough => Color::BLUE,
                AdminLevel::Neighborhood => Color::YELLOW,
                AdminLevel::Parcel => Color::WHITE,
            };
            
            // Draw polygon using Godot drawing API
            _draw_polygon_on_canvas(canvas, &node.boundary, color);
        }
    }
}
```

### Godot Side (GDScript Usage)

```gdscript
# In your main game scene
var admin_system: AdminTreeWrapper

func _ready():
    admin_system = AdminTreeWrapper.new()
    add_child(admin_system)
    
    # Load real-world data
    if admin_system.load_from_osm("res://data/seattle.osm.pbf"):
        print("Administrative boundaries loaded successfully!")
    
    # Query zone under player cursor
    var mouse_pos = get_global_mouse_position()
    var zone = admin_system.get_zone_at_position(mouse_pos.x, mouse_pos.y)
    
    if zone:
        print("Current Zone: ", zone.name)
        print("Level: ", zone.level)
        print("Happiness: ", zone.happiness)
        
        # Show applicable rules
        var rules = admin_system.get_effective_rules(zone.id)
        for rule_key in rules:
            print("  ", rule_key, ": ", rules[rule_key].value)
    
    # Visualize all boundaries
    admin_system.draw_boundaries($CanvasLayer)
```

---

## Implementation Roadmap

### Phase 1: Core Rust Library ✅
- [x] Administrative tree data structures
- [ ] OSM boundary parser (`osm_parser.rs`)
- [ ] GMNS bridge module (`gmns_bridge.rs`)
- [ ] Unit tests for rule resolution

### Phase 2: GDExtension Bindings
- [ ] Set up `godot-rust` project
- [ ] Implement `AdminTreeWrapper` class
- [ ] Create Godot demo scene
- [ ] Test boundary visualization

### Phase 3: Data Pipeline
- [ ] Download OSM .pbf for test region (e.g., Seattle)
- [ ] Parse and validate administrative boundaries
- [ ] Export sample GMNS network with driveways/garages
- [ ] Create test scenarios

### Phase 4: Game Integration
- [ ] Connect to existing Godot vehicle system
- [ ] Integrate with zoning/building growth logic
- [ ] Add UI for rule display and editing
- [ ] Implement "policy change" gameplay mechanic

---

## Next Steps

1. **Install Rust toolchain** on your development machine
2. **Clone the repository** and review `src/admin_tree.rs`
3. **Choose a test region** (city/county with good OSM coverage)
4. **Decide integration approach**:
   - Option A: Start with Godot prototype (existing project)
   - Option B: Build full Rust+GDExtension pipeline first

Would you like me to:
- Generate the OSM parser implementation?
- Create the GDExtension boilerplate?
- Draft a sample GMNS file with driveway/garage extensions?
