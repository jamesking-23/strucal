use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Formula {
    pub id: &'static str, pub name: &'static str, pub standard: &'static str,
    pub clause: &'static str, pub module_version: &'static str, pub verification: &'static str,
}

/// Central formula registry. `verification` stays "PENDING INDEPENDENT REVIEW" until the release gate is met.
pub fn formulas() -> Vec<Formula> {
    let v = "PENDING INDEPENDENT REVIEW";
    vec![
        Formula { id: "EC2-RECT-STRESS-BLOCK", name: "Rectangular stress block flexure", standard: "EN 1992-1-1:2004", clause: "3.1.7, 6.1", module_version: "0.1.0", verification: v },
        Formula { id: "EC2-FCTM", name: "Mean tensile strength", standard: "EN 1992-1-1:2004", clause: "Table 3.1", module_version: "0.1.0", verification: v },
        Formula { id: "EC2-ASMIN", name: "Minimum tension reinforcement", standard: "EN 1992-1-1:2004", clause: "9.2.1.1(1)", module_version: "0.1.0", verification: v },
        Formula { id: "EC2-ASMAX", name: "Maximum reinforcement", standard: "EN 1992-1-1:2004", clause: "9.2.1.1(3)", module_version: "0.1.0", verification: v },
        Formula { id: "EC2-VRDMAX", name: "Shear, compression strut resistance", standard: "EN 1992-1-1:2004", clause: "6.2.3(3) Eq 6.9", module_version: "0.1.0", verification: v },
        Formula { id: "EC2-ASW", name: "Shear links, variable strut inclination", standard: "EN 1992-1-1:2004", clause: "6.2.3(3) Eq 6.8", module_version: "0.1.0", verification: v },
    ]
}
