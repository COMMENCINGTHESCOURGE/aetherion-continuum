pub mod bridge;
pub mod dsl;
pub mod pipeline;
pub mod proof;
pub mod emergence;
pub mod emergence_utils;
pub mod field;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use pyo3::prelude::*;
use dsl::field_dsl::{Parser, WgslGenerator};
use proof::conservation_proof::{ProofChain, ConservationProof, ProofBundle};
use std::fs;

/// Compiles a `#field` DSL script into a WGSL compute shader.
#[cfg(not(target_arch = "wasm32"))]
#[pyfunction]
fn compile_dsl(source: &str) -> PyResult<String> {
    let mut parser = Parser::new(source);
    match parser.parse() {
        Ok(fields) => {
            let wgsl = WgslGenerator::generate(&fields);
            Ok(wgsl)
        }
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!("DSL Parse Error: {}", e))),
    }
}

/// Verifies a CRDT-logged invariant proof chain JSON file.
#[cfg(not(target_arch = "wasm32"))]
#[pyfunction]
fn verify_proof_chain(file_path: &str) -> PyResult<bool> {
    let json_data = fs::read_to_string(file_path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(format!("Failed to read proof file: {}", e)))?;
    
    let bundle: ProofBundle = serde_json::from_str(&json_data)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid proof JSON: {}", e)))?;

    match ProofChain::verify_chain(&bundle.chain) {
        Ok(valid) => Ok(valid),
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!("Verification failed: {}", e))),
    }
}

/// A Python module implemented in Rust for Aetherion-Continuum.
#[cfg(not(target_arch = "wasm32"))]
#[pymodule]
fn aetherion_continuum(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_dsl, m)?)?;
    m.add_function(wrap_pyfunction!(verify_proof_chain, m)?)?;
    Ok(())
}
