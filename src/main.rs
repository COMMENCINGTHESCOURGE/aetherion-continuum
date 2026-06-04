// ═══════════════════════════════════════════════════════════════
// AETHERION-CONTINUUM — Entry Point
// ═══════════════════════════════════════════════════════════════

mod pipeline;
mod dsl;
mod proof;
mod bridge;

use std::env;

fn main() {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║  AETHERION-CONTINUUM v0.1.0                   ║");
    println!("║  Field-Native Conservation Simulation Engine  ║");
    println!("╚═══════════════════════════════════════════════╝");
    println!();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--compile-dsl" {
        // Compile .field DSL files to WGSL
        if args.len() < 3 {
            eprintln!("Usage: aetherion --compile-dsl <input.field>");
            return;
        }
        let source = std::fs::read_to_string(&args[2]).expect("Failed to read DSL file");
        let mut parser = dsl::field_dsl::Parser::new(&source);
        match parser.parse() {
            Ok(fields) => {
                let wgsl = dsl::field_dsl::WgslGenerator::generate(&fields);
                let out_path = args[2].replace(".field", ".wgsl");
                std::fs::write(&out_path, wgsl).expect("Failed to write WGSL output");
                println!("Compiled {} -> {}", args[2], out_path);
            }
            Err(e) => eprintln!("DSL parse error: {}", e),
        }
    } else if args.len() > 1 && args[1] == "--verify-proofs" {
        // Verify conservation proof chain
        if args.len() < 3 {
            eprintln!("Usage: aetherion --verify-proofs <proof_chain.json>");
            return;
        }
        let data = std::fs::read_to_string(&args[2]).expect("Failed to read proof file");
        let bundle: proof::conservation_proof::ProofBundle =
            serde_json::from_str(&data).expect("Invalid proof format");
        match proof::conservation_proof::ProofChain::verify_chain(&bundle.chain) {
            Ok(true) => println!("Proof chain VALID — {} frames, {} corrections, invariance {:.1}%",
                bundle.metadata.total_frames, bundle.metadata.total_corrections,
                bundle.metadata.invariance_ratio * 100.0),
            Ok(false) => println!("Proof chain INVALID"),
            Err(e) => eprintln!("Verification error: {}", e),
        }
    } else {
        println!("Starting GPU compute loop...");
        pollster::block_on(pipeline::zero_sync_dispatch::run());
    }
}
