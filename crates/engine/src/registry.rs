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
        Formula { id: "EC2-SLENDER-LIM", name: "Limiting slenderness", standard: "EN 1992-1-1:2004", clause: "5.8.3.1 Eq 5.13N", module_version: "0.3.0", verification: v },
        Formula { id: "EC2-NOM-CURV", name: "Second order, nominal curvature", standard: "EN 1992-1-1:2004", clause: "5.8.8", module_version: "0.3.0", verification: v },
        Formula { id: "EC2-NM-INTERACTION", name: "N-M interaction by strain compatibility", standard: "EN 1992-1-1:2004", clause: "3.1.7, 6.1", module_version: "0.3.0", verification: v },
        Formula { id: "EC2-COL-DETAIL", name: "Column reinforcement limits and links", standard: "EN 1992-1-1:2004", clause: "9.5.2, 9.5.3", module_version: "0.3.0", verification: v },
        Formula { id: "EC1991-STAIR-LOAD", name: "Stair self-weight on plan", standard: "EN 1991-1-1:2002", clause: "Table A.1", module_version: "0.4.0", verification: v },
        Formula { id: "EC2-WALL-DETAIL", name: "Wall reinforcement limits", standard: "EN 1992-1-1:2004", clause: "9.6.2, 9.6.3", module_version: "0.5.0", verification: v },
        Formula { id: "EC0-ULS-6.10AB", name: "ULS combinations 6.10a/6.10b", standard: "EN 1990:2002", clause: "6.4.3.2 Eq 6.10a/b", module_version: "0.6.0", verification: v },
        Formula { id: "EC0-SLS", name: "SLS characteristic, frequent, quasi-permanent combinations", standard: "EN 1990:2002", clause: "6.5.3 Eq 6.14b, 6.15b, 6.16b", module_version: "0.6.0", verification: v },
        Formula { id: "EC2-VRDC", name: "Shear resistance without shear reinforcement", standard: "EN 1992-1-1:2004", clause: "6.2.2(1) Eq 6.2a/6.3N", module_version: "0.2.0", verification: v },
        Formula { id: "EC2-SPAN-DEPTH", name: "Basic span/effective depth ratio", standard: "EN 1992-1-1:2004", clause: "7.4.2 Eq 7.16a/b", module_version: "0.2.0", verification: v },
        Formula { id: "EC2-PUNCH-MAX", name: "Punching, maximum resistance at column face", standard: "EN 1992-1-1:2004", clause: "6.4.5(3) Eq 6.53", module_version: "0.2.0", verification: v },
        Formula { id: "EC2-PUNCH-C", name: "Punching resistance without shear reinforcement", standard: "EN 1992-1-1:2004", clause: "6.4.4(1) Eq 6.47", module_version: "0.2.0", verification: v },
        Formula { id: "EC1997-BEARING-ALLOW", name: "Bearing by allowable pressure", standard: "EN 1997-1:2004", clause: "6.5.2", module_version: "0.2.0", verification: v },
        Formula { id: "EC0-ULS-6.10", name: "ULS combination, persistent situation", standard: "EN 1990:2002", clause: "6.4.3.2 Eq 6.10", module_version: "0.2.0", verification: v },
    ]
}
