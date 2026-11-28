use serde::Serialize;
use sha2::{Digest, Sha256};

/// One line of working. `formula`/`substitution`/`result` use a tiny markup: `x_{sub}` and `x^{sup}`.
#[derive(Serialize, Clone, Debug)]
pub struct Step {
    pub section: String,
    pub title: String,
    pub reference: String,
    pub formula: String,
    pub substitution: String,
    pub result: String,
    pub comment: String,
    /// plain-text renderings for UIs that cannot show sub/superscripts
    pub calculation: String,
    pub output: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct Trace {
    pub steps: Vec<Step>,
}

/// Strip the markup to plain text (unicode superscripts where possible).
pub fn plain(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    let (mut o, mut i) = (String::new(), 0);
    while i < c.len() {
        if (c[i] == '_' || c[i] == '^') && i + 1 < c.len() && c[i + 1] == '{' {
            if let Some(j) = c[i + 2..].iter().position(|&x| x == '}') {
                let inner: String = c[i + 2..i + 2 + j].iter().collect();
                if c[i] == '^' {
                    match inner.as_str() {
                        "2" => o.push('²'),
                        "3" => o.push('³'),
                        "6" => o.push('⁶'),
                        _ => {
                            o.push_str("^(");
                            o.push_str(&inner);
                            o.push(')');
                        }
                    }
                } else {
                    o.push_str(&inner);
                }
                i += j + 3;
                continue;
            }
        }
        o.push(c[i]);
        i += 1;
    }
    o
}

impl Trace {
    pub fn new() -> Self {
        Trace { steps: vec![] }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn step(&mut self, section: &str, title: &str, reference: &str, formula: &str, subst: &str, result: &str, comment: &str) {
        let mut calc = plain(formula);
        if !subst.is_empty() {
            if !calc.is_empty() {
                calc.push('\n');
            }
            calc.push_str(&plain(subst));
        }
        self.steps.push(Step {
            section: section.into(),
            title: title.into(),
            reference: reference.into(),
            formula: formula.into(),
            substitution: subst.into(),
            result: result.into(),
            comment: comment.into(),
            calculation: calc,
            output: plain(result),
        });
    }
    /// SHA-256 over the ordered trace; identical inputs yield an identical hash.
    pub fn hash(&self) -> String {
        let mut h = Sha256::new();
        for s in &self.steps {
            for f in [&s.section, &s.title, &s.reference, &s.formula, &s.substitution, &s.result] {
                h.update(f.as_bytes());
                h.update(b"|");
            }
            h.update(b"\n");
        }
        h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
    }
}
