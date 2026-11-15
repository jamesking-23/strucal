use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize, Clone, Debug)]
pub struct Step {
    pub reference: String,
    pub calculation: String,
    pub output: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Trace { pub steps: Vec<Step> }

impl Trace {
    pub fn new() -> Self { Trace { steps: vec![] } }
    pub fn push(&mut self, reference: &str, calculation: impl Into<String>, output: impl Into<String>) {
        self.steps.push(Step { reference: reference.into(), calculation: calculation.into(), output: output.into() });
    }
    /// SHA-256 over the ordered trace; identical inputs yield an identical hash.
    pub fn hash(&self) -> String {
        let mut h = Sha256::new();
        for s in &self.steps { h.update(s.reference.as_bytes()); h.update(b"|"); h.update(s.calculation.as_bytes()); h.update(b"|"); h.update(s.output.as_bytes()); h.update(b"\n"); }
        h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
    }
}
