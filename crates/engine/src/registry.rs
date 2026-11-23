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
        Formula { id: "EC3-CLASS", name: "Cross-section classification", standard: "EN 1993-1-1:2005", clause: "5.5, Table 5.2", module_version: "0.7.0", verification: v },
        Formula { id: "EC3-MCRD", name: "Cross-section bending resistance", standard: "EN 1993-1-1:2005", clause: "6.2.5", module_version: "0.7.0", verification: v },
        Formula { id: "EC3-VPLRD", name: "Plastic shear resistance and bending-shear interaction", standard: "EN 1993-1-1:2005", clause: "6.2.6, 6.2.8", module_version: "0.7.0", verification: v },
        Formula { id: "EC3-LTB-ROLLED", name: "LTB reduction factor, rolled sections", standard: "EN 1993-1-1:2005", clause: "6.3.2.3", module_version: "0.7.0", verification: "C1/C2 PENDING SOURCE CHECK" },
        Formula { id: "EC3-NCRD", name: "Cross-section compression resistance", standard: "EN 1993-1-1:2005", clause: "6.2.4", module_version: "0.8.0", verification: v },
        Formula { id: "EC3-FLEX-BUCKLING", name: "Flexural buckling resistance", standard: "EN 1993-1-1:2005", clause: "6.3.1, Tables 6.1/6.2", module_version: "0.8.0", verification: v },
        Formula { id: "EC7-RANKINE-KA", name: "Rankine active earth pressure", standard: "EN 1997-1:2004", clause: "Annex C.1", module_version: "0.9.0", verification: v },
        Formula { id: "EC7-SLIDING", name: "Sliding resistance of a gravity/cantilever wall", standard: "EN 1997-1:2004", clause: "6.5.3, 9.7.4 (DA1 recommended factors)", module_version: "0.9.0", verification: v },
        Formula { id: "EC7-OVERTURNING", name: "Overturning (EQU) and base pressure", standard: "EN 1997-1:2004", clause: "2.4.7.2, 9.7.5, 6.5.2", module_version: "0.9.0", verification: v },
        Formula { id: "RAFT-RIGID-STRIP", name: "Conventional rigid method, column-line strip statics", standard: "Equilibrium (no code clause)", clause: "Statics of V(x), M(x)", module_version: "0.10.0", verification: "METHOD PENDING REVIEW" },
        Formula { id: "EC2-PUNCH-BETA", name: "Punching, interior column eccentricity factor", standard: "EN 1992-1-1:2004", clause: "6.4.3(6) beta = 1.15", module_version: "0.10.0", verification: v },
        Formula { id: "EC3-8-BOLT-SHEAR", name: "Bolt shear resistance", standard: "EN 1993-1-8:2005", clause: "Table 3.4", module_version: "0.11.0", verification: v },
        Formula { id: "EC3-8-BOLT-BEARING", name: "Bolt bearing resistance", standard: "EN 1993-1-8:2005", clause: "Table 3.4", module_version: "0.11.0", verification: v },
        Formula { id: "EC3-8-SPACING", name: "Edge and end distances, spacing", standard: "EN 1993-1-8:2005", clause: "Table 3.3", module_version: "0.11.0", verification: v },
        Formula { id: "EC3-8-LONG-JOINT", name: "Long joint reduction", standard: "EN 1993-1-8:2005", clause: "3.8", module_version: "0.11.0", verification: v },
        Formula { id: "EC3-1-NET-SECTION", name: "Plate gross and net section resistance", standard: "EN 1993-1-1:2005", clause: "6.2.3", module_version: "0.11.0", verification: v },
        Formula { id: "EC3-8-WELD-DIR", name: "Fillet weld, directional method", standard: "EN 1993-1-8:2005", clause: "4.5.3.2", module_version: "0.11.0", verification: v },
        Formula { id: "EC3-8-WELD-SIMPLE", name: "Fillet weld, simplified method", standard: "EN 1993-1-8:2005", clause: "4.5.3.3, Table 4.1", module_version: "0.11.0", verification: v },
        Formula { id: "PROJ-DEPENDENCY", name: "Project dependency graph and change propagation", standard: "StruCal specification Part H", clause: "Topological order, stale marking, impact report", module_version: "0.12.0", verification: "SOFTWARE FUNCTION - NOT AN ENGINEERING FORMULA" },
        Formula { id: "EC5-TIMBER-DESIGN-STRENGTH", name: "Timber design strengths (k_mod, gamma_M, k_sys, k_h)", standard: "EN 1995-1-1 (as printed in supplied source, UK NA)", clause: "Eq 11.2, 11.3, Tables 11.4-11.5", module_version: "0.16.0", verification: "CHECKED AGAINST WORKED EXAMPLES; NA PENDING" },
        Formula { id: "EC5-TIMBER-LTB", name: "Timber lateral buckling k_crit", standard: "EN 1995-1-1 (as printed in supplied source)", clause: "Eq 11.18-11.24, Table 11.11", module_version: "0.16.0", verification: "SOFTWOOD FORMULA; PENDING REVIEW" },
        Formula { id: "EC5-TIMBER-SHEAR-NOTCH", name: "Timber shear and end notch factor k_v", standard: "EN 1995-1-1 (as printed in supplied source)", clause: "Eq 11.25-11.31", module_version: "0.16.0", verification: "CHECKED AGAINST WORKED EXAMPLE 11.2" },
        Formula { id: "EC5-TIMBER-BEARING", name: "Compression perpendicular to the grain, k_c,90", standard: "EN 1995-1-1 (as printed in supplied source)", clause: "Eq 11.32-11.34", module_version: "0.16.0", verification: "CHECKED AGAINST WORKED EXAMPLE 11.1" },
        Formula { id: "EC5-TIMBER-DEFLECTION", name: "Timber final deflection with creep", standard: "EN 1995-1-1 (as printed in supplied source)", clause: "Eq 11.8-11.10, Table 11.9", module_version: "0.16.0", verification: "CHECKED AGAINST WORKED EXAMPLE 11.1" },
        Formula { id: "EC5-TIMBER-COLUMN", name: "Timber column buckling and combined bending", standard: "EN 1995-1-1 (as printed in supplied source)", clause: "Eq 11.35-11.50", module_version: "0.16.0", verification: "CHECKED AGAINST WORKED EXAMPLES 11.4, 11.5" },
        Formula { id: "EC2-VRDC", name: "Shear resistance without shear reinforcement", standard: "EN 1992-1-1:2004", clause: "6.2.2(1) Eq 6.2a/6.3N", module_version: "0.2.0", verification: v },
        Formula { id: "EC2-SPAN-DEPTH", name: "Basic span/effective depth ratio", standard: "EN 1992-1-1:2004", clause: "7.4.2 Eq 7.16a/b", module_version: "0.2.0", verification: v },
        Formula { id: "EC2-PUNCH-MAX", name: "Punching, maximum resistance at column face", standard: "EN 1992-1-1:2004", clause: "6.4.5(3) Eq 6.53", module_version: "0.2.0", verification: v },
        Formula { id: "EC2-PUNCH-C", name: "Punching resistance without shear reinforcement", standard: "EN 1992-1-1:2004", clause: "6.4.4(1) Eq 6.47", module_version: "0.2.0", verification: v },
        Formula { id: "EC1997-BEARING-ALLOW", name: "Bearing by allowable pressure", standard: "EN 1997-1:2004", clause: "6.5.2", module_version: "0.2.0", verification: v },
        Formula { id: "EC0-ULS-6.10", name: "ULS combination, persistent situation", standard: "EN 1990:2002", clause: "6.4.3.2 Eq 6.10", module_version: "0.2.0", verification: v },
    ]
}
