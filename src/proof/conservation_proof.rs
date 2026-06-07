// ═══════════════════════════════════════════════════════════════
// AETHERION-CONTINUUM — Conservation Proof Exporter
// CRDT-logged invariant verification.
// Exports JSON proof files per frame: mass drift, energy drift,
// momentum vector, correction count, invariant violations.
// Verifiable off-chain. No trust required.
// ═══════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use chrono::Utc;

// ── Proof Types ────────────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConservationProof {
    pub frame: u64,
    pub timestamp: String,
    pub mass_drift: f64,
    pub energy_drift: f64,
    pub momentum_drift: [f64; 3],
    pub corrections_applied: u32,
    pub cells_corrected: u32,
    pub invariant_violations: u32,
    pub hash_prev: String,     // SHA256 of previous proof (CRDT chain)
    pub hash_self: String,     // SHA256 of this proof
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorrectionEntry {
    pub cell_idx: u32,
    pub pre_mass: f32,
    pub post_mass: f32,
    pub divergence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofBundle {
    pub chain: Vec<ConservationProof>,
    pub corrections: Vec<Vec<CorrectionEntry>>,  // per-frame corrections
    pub metadata: ProofMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub engine_version: String,
    pub total_frames: u64,
    pub total_corrections: u64,
    pub mean_mass_drift: f64,
    pub max_mass_drift: f64,
    pub invariance_ratio: f64,  // frames_with_zero_violations / total_frames
}

// ── CRDT Chain ─────────────────────────────────────────────────
pub struct ProofChain {
    proofs: Vec<ConservationProof>,
    corrections: Vec<Vec<CorrectionEntry>>,
    output_dir: PathBuf,
    last_hash: String,
}

impl ProofChain {
    pub fn new(output_dir: PathBuf) -> Self {
        fs::create_dir_all(&output_dir).ok();
        ProofChain {
            proofs: Vec::new(),
            corrections: Vec::new(),
            output_dir,
            last_hash: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        }
    }

    /// Append a frame's conservation proof, chaining via SHA256.
    pub fn append_frame(
        &mut self,
        frame: u64,
        mass_drift: f64,
        energy_drift: f64,
        momentum_drift: [f64; 3],
        corrections_applied: u32,
        cells_corrected: u32,
        frame_corrections: Vec<CorrectionEntry>,
    ) {
        let timestamp = Utc::now().to_rfc3339();
        let invariant_violations = if corrections_applied > 0 { 1 } else { 0 };

        // Build proof without hash_self first, then compute hash
        let hash_self = Self::compute_hash(frame, &timestamp, mass_drift, energy_drift,
                                            momentum_drift, &self.last_hash);

        let proof = ConservationProof {
            frame,
            timestamp,
            mass_drift,
            energy_drift,
            momentum_drift,
            corrections_applied,
            cells_corrected,
            invariant_violations,
            hash_prev: self.last_hash.clone(),
            hash_self,
            verified: invariant_violations == 0,
        };

        self.last_hash = proof.hash_self.clone();
        self.proofs.push(proof);
        self.corrections.push(frame_corrections);
    }

    /// Export full proof chain to JSON file.
    pub fn export(&self) -> std::io::Result<PathBuf> {
        let total_frames = self.proofs.len() as u64;
        let total_corrections: u64 = self.proofs.iter().map(|p| p.corrections_applied as u64).sum();
        let mean_mass_drift: f64 = self.proofs.iter().map(|p| p.mass_drift.abs()).sum::<f64>()
            / (total_frames.max(1) as f64);
        let max_mass_drift: f64 = self.proofs.iter()
            .map(|p| p.mass_drift.abs())
            .fold(0.0, f64::max);
        let invariance_ratio: f64 = self.proofs.iter().filter(|p| p.invariant_violations == 0).count() as f64
            / (total_frames.max(1) as f64);

        let bundle = ProofBundle {
            chain: self.proofs.clone(),
            corrections: self.corrections.clone(),
            metadata: ProofMetadata {
                engine_version: "aetherion-continuum v0.1.0".into(),
                total_frames,
                total_corrections,
                mean_mass_drift,
                max_mass_drift,
                invariance_ratio,
            },
        };

        let path = self.output_dir.join(format!("proof_chain_{}.json", Utc::now().format("%Y%m%d_%H%M%S")));
        let json = serde_json::to_string_pretty(&bundle)?;
        let mut file = File::create(&path)?;
        file.write_all(json.as_bytes())?;
        println!("Conservation proof exported: {} | {} frames | drift {:.6} | invariance {:.1}%",
            path.display(), total_frames, mean_mass_drift, invariance_ratio * 100.0);
        Ok(path)
    }

    /// Verify a proof chain for integrity (CRDT link validation).
    pub fn verify_chain(proofs: &[ConservationProof]) -> Result<bool, String> {
        if proofs.is_empty() { return Ok(true); }

        let mut prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        for proof in proofs {
            if proof.hash_prev != prev_hash {
                return Err(format!("Chain break at frame {}: expected prev_hash {}, got {}",
                    proof.frame, prev_hash, proof.hash_prev));
            }
            let computed = Self::compute_hash(proof.frame, &proof.timestamp,
                proof.mass_drift, proof.energy_drift, proof.momentum_drift, &prev_hash);
            if computed != proof.hash_self {
                return Err(format!("Hash mismatch at frame {}: expected {}, computed {}",
                    proof.frame, proof.hash_self, computed));
            }
            prev_hash = proof.hash_self.clone();
        }
        Ok(true)
    }

    fn compute_hash(
        frame: u64, timestamp: &str,
        mass_drift: f64, energy_drift: f64,
        momentum_drift: [f64; 3],
        prev_hash: &str,
    ) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        // Use big-endian for cross-platform consistency
        hasher.update(frame.to_be_bytes());
        hasher.update(timestamp.as_bytes());
        hasher.update(mass_drift.to_be_bytes());
        hasher.update(energy_drift.to_be_bytes());
        for v in momentum_drift { hasher.update(v.to_be_bytes()); }
        hasher.update(prev_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// ── Live Proof Monitor ─────────────────────────────────────────
pub struct ProofMonitor {
    chain: ProofChain,
    export_interval: u64,  // export every N frames
    frames_since_export: u64,
}

impl ProofMonitor {
    pub fn new(output_dir: PathBuf, export_interval: u64) -> Self {
        ProofMonitor {
            chain: ProofChain::new(output_dir),
            export_interval,
            frames_since_export: 0,
        }
    }

    pub fn record_frame(&mut self, mass_drift: f64, energy_drift: f64,
                        momentum: [f64; 3], corrections: u32, cells: u32,
                        frame_corrections: Vec<CorrectionEntry>) {
        let frame = self.chain.proofs.len() as u64;
        self.chain.append_frame(frame, mass_drift, energy_drift, momentum,
                                corrections, cells, frame_corrections);
        self.frames_since_export += 1;

        if self.frames_since_export >= self.export_interval {
            self.chain.export().ok();
            self.frames_since_export = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_chain_integrity() {
        let dir = std::env::temp_dir().join("aetherion_proof_test");
        let mut chain = ProofChain::new(dir.clone());
        chain.append_frame(0, 0.0001, 0.00005, [0.0, 0.0, 0.0], 0, 0, vec![]);
        chain.append_frame(1, 0.0002, 0.00003, [0.0001, 0.0, 0.0], 1, 5, vec![CorrectionEntry {
            cell_idx: 42, pre_mass: 0.5, post_mass: 0.4998, divergence: 0.001,
        }]);

        let valid = ProofChain::verify_chain(&chain.proofs).unwrap();
        assert!(valid);
        fs::remove_dir_all(&dir).ok();
    }
}

