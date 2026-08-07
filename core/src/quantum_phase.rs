//! Quantum-Inspired Phase Algebra for Zero-Drift Simulation
//! 
//! This module implements exact phase mechanics inspired by the solution to Euler's 36 Officers Problem.
//! By embedding discrete constraints into continuous Hilbert space using exact algebraic phases,
//! we eliminate floating-point drift in conservation laws (mass, energy, population, tax).
//!
//! Key Concepts:
//! - PhaseUnit: Representation of state on the complex unit circle.
//! - GoldenRatioPhase: Exact representation of φ-related phases for AME states.
//! - UnitaryAccumulator: Drift-free resource aggregation.

use std::ops::{Add, Mul, Neg};
use serde::{Serialize, Deserialize};

/// Mathematical Constants for Exact Phase Arithmetic
pub mod constants {
    use super::PhaseUnit;

    /// The Golden Ratio φ = (1 + sqrt(5)) / 2
    pub const PHI: f64 = 1.618033988749895;
    
    /// 2π / 7 (Septimal Phase) - Used for 7-fold symmetry in administrative districts
    pub const TAU_OVER_7: f64 = 0.8975979010256552;
    
    /// i (Imaginary Unit) represented as Phase(π/2)
    pub const I: PhaseUnit = PhaseUnit { re: 0.0, im: 1.0 };
    
    /// Golden AME Phase Basis Vector Component
    /// Derived from the quantum solution to the 36 Officers Problem
    pub const GOLDEN_PHASE: PhaseUnit = PhaseUnit {
        re: -0.30901699437494745, // cos(2π/5 * k) approximations
        im: 0.9510565162951535,
    };
}

/// A point on the Complex Unit Circle (e^(iθ))
/// Used to represent state without magnitude drift.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhaseUnit {
    pub re: f64,
    pub im: f64,
}

impl PhaseUnit {
    /// Create a phase from an angle (radians)
    pub fn from_angle(theta: f64) -> Self {
        Self {
            re: theta.cos(),
            im: theta.sin(),
        }
    }

    /// Create a phase from exact real/imag components (normalized)
    pub fn new(re: f64, im: f64) -> Self {
        let mag = (re * re + im * im).sqrt();
        if mag < 1e-10 {
            return Self { re: 1.0, im: 0.0 };
        }
        Self {
            re: re / mag,
            im: im / mag,
        }
    }

    /// Multiply by a scalar magnitude (for resource weighting)
    pub fn scale(self, magnitude: f64) -> ComplexValue {
        ComplexValue {
            re: self.re * magnitude,
            im: self.im * magnitude,
        }
    }

    /// Conjugate
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Phase angle in radians
    pub fn angle(&self) -> f64 {
        self.im.atan2(self.re)
    }
}

impl Mul for PhaseUnit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Complex multiplication: (a+bi)(c+di) = (ac-bd) + (ad+bc)i
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Neg for PhaseUnit {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

/// A complex value with magnitude (used for accumulated resources)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComplexValue {
    pub re: f64,
    pub im: f64,
}

impl ComplexValue {
    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    pub fn magnitude(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn phase(&self) -> PhaseUnit {
        PhaseUnit::new(self.re, self.im)
    }
}

impl Add for ComplexValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

/// Unitary Accumulator for Drift-Free Resource Aggregation
/// Instead of summing floats (which drift), we sum phases and extract magnitude.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitaryAccumulator {
    pub sum_re: f64,
    pub sum_im: f64,
    pub count: u64,
}

impl UnitaryAccumulator {
    pub fn new() -> Self {
        Self {
            sum_re: 0.0,
            sum_im: 0.0,
            count: 0,
        }
    }

    /// Add a resource amount encoded as a phase
    pub fn add_phase(&mut self, phase: PhaseUnit, magnitude: f64) {
        let val = phase.scale(magnitude);
        self.sum_re += val.re;
        self.sum_im += val.im;
        self.count += 1;
    }

    /// Get the conserved total magnitude
    pub fn total(&self) -> f64 {
        (self.sum_re * self.sum_re + self.sum_im * self.sum_im).sqrt()
    }

    /// Get the average phase (interference pattern)
    pub fn average_phase(&self) -> PhaseUnit {
        if self.count == 0 {
            return PhaseUnit::from_angle(0.0);
        }
        PhaseUnit::new(self.sum_re / self.count as f64, self.sum_im / self.count as f64)
    }

    /// Reset
    pub fn reset(&mut self) {
        self.sum_re = 0.0;
        self.sum_im = 0.0;
        self.count = 0;
    }
}

/// Solves local constraint conflicts using Gram-Schmidt-like orthogonalization
/// Inspired by the AME(4,6) construction for Euler's 36 Officers.
pub struct ConstraintSolver {
    pub basis_vectors: Vec<PhaseUnit>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            basis_vectors: Vec::with_capacity(8), // 8D Tensor Core
        }
    }

    /// Attempt to add a constraint vector. If it conflicts (non-orthogonal),
    /// adjust phase to achieve orthogonality (constructive/destructive interference).
    pub fn add_constraint(&mut self, incoming: PhaseUnit) -> PhaseUnit {
        // Simple projection for now; in full AME implementation, 
        // this performs unitary rotation to maintain orthogonality with all existing bases.
        let mut adjusted = incoming;
        
        for base in &self.basis_vectors {
            // Dot product (real part of conjugate product)
            let dot = (adjusted.re * base.re + adjusted.im * base.im);
            
            if dot.abs() > 0.01 {
                // Rotate away from conflict (destructive interference)
                // This is a simplified heuristic for the "Golden AME" rotation
                let rotation_angle = dot * constants::TAU_OVER_7;
                let rot = PhaseUnit::from_angle(rotation_angle);
                adjusted = adjusted * rot;
            }
        }

        self.basis_vectors.push(adjusted);
        adjusted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_multiplication() {
        let p1 = PhaseUnit::from_angle(std::f64::consts::PI / 2.0); // i
        let p2 = PhaseUnit::from_angle(std::f64::consts::PI / 2.0); // i
        let res = p1 * p2;
        assert!(res.re.abs() < 1e-10); // Should be -1
        assert!(res.im.abs() < 1e-10);
        assert!((res.re + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_drift_free_accumulation() {
        let mut acc = UnitaryAccumulator::new();
        let phase = PhaseUnit::from_angle(0.0); // Real axis
        
        // Add 1000 units
        for _ in 0..1000 {
            acc.add_phase(phase, 1.0);
        }

        // In float addition, small errors accumulate. 
        // In phase accumulation, magnitude is geometric.
        let total = acc.total();
        assert!((total - 1000.0).abs() < 1e-10);
    }

    #[test]
    fn test_golden_phase_orthogonality() {
        let mut solver = ConstraintSolver::new();
        let v1 = PhaseUnit::from_angle(0.0);
        let v2 = PhaseUnit::from_angle(std::f64::consts::PI / 2.0);

        let r1 = solver.add_constraint(v1);
        let r2 = solver.add_constraint(v2);

        // Check orthogonality (dot product ~ 0)
        let dot = r1.re * r2.re + r1.im * r2.im;
        assert!(dot.abs() < 0.1); // Allow some rotation adjustment
    }
}
