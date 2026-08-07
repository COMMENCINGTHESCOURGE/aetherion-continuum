//! OSM Boundary Parser CLI
//! 
//! Usage: cargo run --bin parse_boundaries -- <input.pbf> [output.json]

use aetherion_parser::BoundaryParser;
use std::env;
use std::fs;

fn main() {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: parse_boundaries <input.osm.pbf> [output.json]");
        eprintln!("  input.osm.pbf  - Path to OSM PBF file");
        eprintln!("  output.json    - Optional path to save AdminTree as JSON");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = args.get(2).map(|s| s.as_str());

    println!("🗺️  Parsing OSM boundaries from: {}", input_path);

    let mut parser = BoundaryParser::new();
    
    match parser.parse_pbf(input_path) {
        Ok(tree) => {
            println!("✅ Successfully parsed administrative hierarchy");
            println!("   Counties: {}", tree.nodes.values().filter(|n| matches!(n.level, aetherion_core::AdminLevel::County)).count());
            println!("   Cities: {}", tree.nodes.values().filter(|n| matches!(n.level, aetherion_core::AdminLevel::City)).count());
            println!("   Boroughs: {}", tree.nodes.values().filter(|n| matches!(n.level, aetherion_core::AdminLevel::Borough)).count());
            println!("   Neighborhoods: {}", tree.nodes.values().filter(|n| matches!(n.level, aetherion_core::AdminLevel::Neighborhood)).count());

            // Serialize to JSON (note: boundaries are skipped due to #[serde(skip)])
            if let Some(output) = output_path {
                let json = serde_json::to_string_pretty(&tree)
                    .expect("Failed to serialize AdminTree");
                fs::write(output, json).expect("Failed to write output file");
                println!("💾 Saved AdminTree to: {}", output);
            }
        }
        Err(e) => {
            eprintln!("❌ Error parsing PBF file: {}", e);
            std::process::exit(1);
        }
    }
}
