# StruCal verification register

**Purpose.** Record, per calculation module, what has been checked, by what means, and what is still open. A module is "released" only after a qualified person who did not write it signs the table at the end of its section.

**Honest status today.** Every module is **pending independent review**. The cross-checks below were made by the author of the software (hand calculations and separate scripts). They catch coding errors but are not an independent review, and several test the same method twice.

**Status key**

- A: author cross-checked (own hand calculation or script)
- B: also checked against published section or standard values
- C: no independent cross-check yet

**Applies to all modules**

- Uganda's National Annex is not verified; EN recommended values are used unless stated.
- Formulas were implemented from the Eurocode clauses cited in each trace. The standards were not available in this repository, so clause references and numerical constants must be checked against the licensed documents.
- A PASS applies only to the checks listed and is not an engineer's approval.

**How to regenerate the tables below:** `cargo run -p strucal-engine --example verification_report`. Engine version at generation: 0.15.0.

**Suggested review procedure for each module**

1. Re-derive the default case by hand from the cited clauses (not from the code).
2. Compare every row of the results table and the step-by-step trace in the exported calculation sheet.
3. Vary at least three inputs (a light case, a heavy case and a failing case) and confirm PASS/FAIL changes as expected.
4. Confirm the limits listed under "Known limits" are acceptable for the intended use.
5. Record the outcome in the sign-off table.

## Load combinations (EN 1990) (`combos`)

EN 1990:2002 · Persistent situation, STR (Eq 6.10, 6.10a/b) and SLS combinations of action effects

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Permanent action effect G k | 100 | effect |
| Variable action 1: effect Q k,1 | 50 | effect |
| ψ0 | 0.7 | - |
| ψ1 | 0.5 | - |
| ψ2 | 0.3 | - |
| Variable action 2: effect Q k,2 (0 if unused) | 30 | effect |
| ψ0 | 0.6 | - |
| ψ1 | 0.2 | - |
| ψ2 | 0 | - |
| Variable action 3: effect Q k,3 (0 if unused) | 0 | effect |
| ψ0 | 0.5 | - |
| ψ1 | 0.2 | - |
| ψ2 | 0 | - |

**Engine results**

| Result | Value |
|---|---|
| ULS design value, Eq 6.10 | 237.00  (leading Q1) |
| ULS design value, 6.10a/b | 216.75 |
| Favourable permanent | 100.00 |
| SLS characteristic | 168.00 |
| SLS frequent | 125.00 |
| SLS quasi-permanent | 115.00 |
| Equivalent γ on total (6.10) | 1.317 |

*This module reports design values and has no PASS/FAIL checks.*

Trace steps: 13; trace hash `c5b82c3d98fba5e8e9402d16e5e641e99d3c54a2a6aaa0b743c797bf0d0038a4`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Hand calculation of EN 1990 Eq 6.10, 6.10a/b and SLS combinations for Gk=100, Q1=50 (psi0 0.7), Q2=30 (psi0 0.6): 237.00, 216.75, 168.0, 134.0, 124.0 (matches engine tests).

**Known limits and open issues:** psi factors are user inputs (EN 1990 Table A1.1 / National Annex). EQU/GEO sets, accidental and seismic not implemented.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete beam (`beam`)

EN 1992-1-1:2004 · Rectangular section, sagging moment and shear

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Width b | 300 | mm |
| Depth h | 500 | mm |
| Cover c | 30 | mm |
| Link φ | 8 | mm |
| Tension bar φ | 20 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |
| Moment M Ed | 200 | kNm |
| Shear V Ed | 120 | kN |

**Engine results**

| Result | Value |
|---|---|
| Effective depth d | 452.0 mm |
| Required tension steel As,req | 1118 mm² |
| Provided tension steel As,prov | 1257 mm²  (4T20) |
| Top steel | 2T12 nominal |
| Links | T8 2-leg @ 325 c/c |
| Lever arm z | 411.5 mm |
| μ / μlim | 0.1632 / 0.2952 |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Flexural resistance | EN 1992-1-1 6.1 | 200.0 kNm | 222.1 kNm | 0.90 | PASS |
| Minimum reinforcement | EN 1992-1-1 9.2.1.1(1) | 204 mm² | 1257 mm² | 0.16 | PASS |
| Maximum reinforcement | EN 1992-1-1 9.2.1.1(3) | 1257 mm² | 6000 mm² | 0.21 | PASS |
| Bar spacing | EN 1992-1-1 8.2(2) | 25 mm min. | 48 mm clear | 0.52 | PASS |
| Shear: compression strut | EN 1992-1-1 6.2.3 Eq 6.9 | 120.0 kN | 444.4 kN | 0.27 | PASS |
| Shear reinforcement | EN 1992-1-1 6.2.3 Eq 6.8, 9.2.2 | 0.271 mm²/mm | 0.309 mm²/mm | 0.88 | PASS |
| Link spacing limit | EN 1992-1-1 9.2.2(6) | 325 mm | 339 mm max | 0.96 | PASS |

Trace steps: 27; trace hash `c42598cc9d59863ff05fe5725f3e247df56f6987a822e4d5567b24c4e5100c49`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Hand calculation d = 452 mm, mu = 0.1632, z = 411.5 mm, As,req = 1118 mm2 for 300x500, C30/37, B500, MEd = 200 kNm. Shear and minimum/maximum steel by formula only.

**Known limits and open issues:** Torsion, crack control, deflection, anchorage, laps, flanged sections not implemented. Uganda National Annex not verified (recommended values used).

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete one-way slab (`slab`)

EN 1992-1-1:2004 · Simply supported single span, 1 m strip

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Span L | 4000 | mm |
| Thickness h | 175 | mm |
| Cover c | 25 | mm |
| Main bar φ | 12 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |
| Superimposed dead gk (excl. self-weight) | 1.5 | kN/m² |
| Imposed qk | 3 | kN/m² |

**Engine results**

| Result | Value |
|---|---|
| Effective depth d | 144.0 mm |
| Design load w Ed | 12.43 kN/m |
| M Ed / V Ed | 24.86 kNm / 24.86 kN |
| Required steel As,req | 418 mm²/m |
| Main reinforcement | T12 @ 250  (452 mm²/m) |
| Secondary | T8 @ 450 |
| Span/depth | 27.8 (limit 44.5) |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Flexural resistance | EN 1992-1-1 6.1 | 24.86 kNm/m | 27.36 kNm/m | 0.91 | PASS |
| Minimum reinforcement | EN 1992-1-1 9.2.1.1(1) | 217 mm²/m | 452 mm²/m | 0.48 | PASS |
| Maximum reinforcement | EN 1992-1-1 9.2.1.1(3) | 452 mm²/m | 7000 mm²/m | 0.06 | PASS |
| Bar spacing (max) | EN 1992-1-1 9.3.1.1(3) | 250 mm | 400 mm max | 0.62 | PASS |
| Bar spacing (clear) | EN 1992-1-1 8.2(2) | 25 mm min | 238 mm clear | 0.11 | PASS |
| Shear (no links) | EN 1992-1-1 6.2.2(1) | 24.9 kN/m | 78.1 kN/m | 0.32 | PASS |
| Deflection (span/depth) | EN 1992-1-1 7.4.2 | 27.8 | 44.5 allowed | 0.62 | PASS |

Trace steps: 16; trace hash `d8f2f4774c5f00d462de2046064c25d9ec19015de0cf4bd832c56f150bc44016`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Hand calculation for 4 m span, h = 175, gk 1.5, qk 3: w = 12.43 kN/m, MEd = 24.9 kNm/m, As,req about 418 mm2/m. Deflection check not independently reproduced.

**Known limits and open issues:** Alpha_cc and the 1.5 modification-factor cap are National Annex values (cap is UK NA) and unverified for Uganda. Continuous/two-way slabs not implemented.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete staircase (waist slab flight) (`stairs`)

EN 1992-1-1:2004 · Simply supported flight, design on the horizontal span, 1 m width

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Horizontal span L (between supports) | 3000 | mm |
| Waist thickness h (perpendicular) | 150 | mm |
| Riser height | 175 | mm |
| Going (tread) | 250 | mm |
| Cover c | 25 | mm |
| Main bar φ | 12 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |
| Finishes gk (per m² of plan) | 1.5 | kN/m² |
| Imposed qk (per m² of plan) | 4 | kN/m² |

**Engine results**

| Result | Value |
|---|---|
| Effective depth d | 119.0 mm |
| Design load w Ed | 17.16 kN/m |
| M Ed / V Ed | 19.30 kNm / 25.74 kN |
| Required steel As,req | 393 mm²/m |
| Main reinforcement | T12 @ 275  (411 mm²/m) |
| Secondary | T8 @ 450 |
| Span/depth | 25.2 (limit 35.6) |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Flexural resistance | EN 1992-1-1 6.1 | 19.30 kNm/m | 20.48 kNm/m | 0.94 | PASS |
| Minimum reinforcement | EN 1992-1-1 9.2.1.1(1) | 179 mm²/m | 411 mm²/m | 0.44 | PASS |
| Maximum reinforcement | EN 1992-1-1 9.2.1.1(3) | 411 mm²/m | 6000 mm²/m | 0.07 | PASS |
| Bar spacing (max) | EN 1992-1-1 9.3.1.1(3) | 275 mm | 400 mm max | 0.69 | PASS |
| Bar spacing (clear) | EN 1992-1-1 8.2(2) | 25 mm min | 263 mm clear | 0.10 | PASS |
| Shear (no links) | EN 1992-1-1 6.2.2(1) | 25.7 kN/m | 64.5 kN/m | 0.40 | PASS |
| Deflection (span/depth) | EN 1992-1-1 7.4.2 | 25.2 | 35.6 allowed | 0.71 | PASS |

Trace steps: 16; trace hash `162332c1a6eb478c3463787152e6a4ef94defd88253309212dcf36ba17da6920`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Hand calculation for span 3 m, waist 150, rise 175, going 250, gk 1.5, qk 4: cos alpha = 0.8192, g_sw = 6.765 kPa on plan, w = 17.158 kN/m, MEd = 19.302 kNm/m, VEd = 25.737 kN/m, As,req = 392.7 mm2/m (matches engine and an automated test). Deflection check shares the slab implementation and is not independently reproduced.

**Known limits and open issues:** Landings, dynamic checks and stair geometry rules not implemented. 2R+G range is a comfort guide only.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete column (`column`)

EN 1992-1-1:2004 · Braced rectangular column, axial load with uniaxial bending, nominal curvature second order

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Width b (out of bending plane) | 300 | mm |
| Depth h (in bending plane) | 400 | mm |
| Column clear height l | 3500 | mm |
| Effective length factor l0/l | 1 | - |
| Cover c | 35 | mm |
| Link φ | 8 | mm |
| Longitudinal bar φ | 20 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |
| Effective creep ratio φef | 1 | - |
| Axial load N Ed (ULS) | 1500 | kN |
| Larger end moment M02 | 80 | kNm |
| Smaller end moment M01 (signed, e.g. -40 for double curvature) | 40 | kNm |

**Engine results**

| Result | Value |
|---|---|
| Slenderness λ / λlim | 30.3 / 27.8 |
| Design moment M Ed | 88.2 kNm  (M2 = 24.2) |
| Moment resistance M Rd | 172.0 kNm |
| Axial capacity N Rd,0 | 2874 kN |
| Longitudinal steel | 4T20  (1257 mm²) |
| Links | T8 @ 300 c/c |
| Steel ratio | 1.05 % |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Axial + bending interaction | EN 1992-1-1 6.1, 5.8.8 | 88.2 kNm | 172.0 kNm | 0.51 | PASS |
| Axial capacity | EN 1992-1-1 6.1 | 1500 kN | 2874 kN | 0.52 | PASS |
| Minimum reinforcement | EN 1992-1-1 9.5.2(2) | 345 mm² | 1257 mm² | 0.27 | PASS |
| Maximum reinforcement | EN 1992-1-1 9.5.2(3) | 1257 mm² | 4800 mm² | 0.26 | PASS |
| Bar clear spacing | EN 1992-1-1 8.2(2) | 25 mm min | 174 mm clear | 0.14 | PASS |
| Link diameter | EN 1992-1-1 9.5.3(1) | 6.0 mm min | 8 mm | 0.75 | PASS |
| Link spacing | EN 1992-1-1 9.5.3(3) | 300 mm | 300 mm max | 1.00 | PASS |

Trace steps: 16; trace hash `537d84dd9376976a54810d8e627ecb07a466125455b527531aa7019bb3701809`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Independent Python re-implementation of the same strain-compatibility method gives MRd = 172.0 kNm for the default case (N = 1500 kN, 2T20 per face). Same method written twice: it checks coding, not the method. Slenderness and nominal-curvature steps not independently reproduced.

**Known limits and open issues:** Biaxial bending, weak-axis slenderness, unbraced frames, shear not implemented. Side bars ignored in resistance.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete wall (per metre length) (`wall`)

EN 1992-1-1:2004 · Braced wall, vertical load with out-of-plane bending, nominal curvature second order

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Wall thickness h | 200 | mm |
| Clear height l | 3000 | mm |
| Effective length factor l0/l | 1 | - |
| Cover c (to horizontal bars) | 30 | mm |
| Vertical bar φ | 12 | mm |
| Horizontal bar φ | 10 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |
| Effective creep ratio φef | 1 | - |
| Vertical load N Ed (ULS) | 600 | kN/m |
| Larger end moment M02 (out of plane) | 12 | kNm/m |
| Smaller end moment M01 (signed) | 6 | kNm/m |

**Engine results**

| Result | Value |
|---|---|
| Slenderness λ / λlim | 52.0 / 56.8 |
| Design moment M Ed | 12.00 kNm/m |
| Moment resistance M Rd | 65.89 kNm/m |
| Axial capacity N Rd,0 | 4213 kN/m |
| Vertical steel | T12 @ 400 each face  (565 mm²/m) |
| Horizontal steel | T10 @ 400 each face |
| Steel ratio | 0.28 % |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Axial + bending interaction | EN 1992-1-1 6.1, 5.8.8 | 12.00 kNm/m | 65.89 kNm/m | 0.18 | PASS |
| Axial capacity | EN 1992-1-1 6.1 | 600 kN/m | 4213 kN/m | 0.14 | PASS |
| Minimum vertical reinforcement | EN 1992-1-1 9.6.2(1) | 400 mm²/m | 565 mm²/m | 0.71 | PASS |
| Maximum vertical reinforcement | EN 1992-1-1 9.6.2(2) | 565 mm²/m | 8000 mm²/m | 0.07 | PASS |
| Vertical bar spacing (max) | EN 1992-1-1 9.6.2(3) | 400 mm | 400 mm max | 1.00 | PASS |
| Vertical bar spacing (clear) | EN 1992-1-1 8.2(2) | 25 mm min | 388 mm clear | 0.06 | PASS |

Trace steps: 15; trace hash `578da923ca7311dcaa48c2eb35834539ca23c5309712dd65fd3d5209f712e897`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** A separately coded strain-compatibility calculation for the 1 m strip (h 200, N = 600 kN/m, T12 at 400 each face) gives MRd = 65.89 kNm/m, matching the engine (same method, coded twice). Slenderness and second-order steps and the vertical/horizontal steel limits are not independently reproduced.

**Known limits and open issues:** In-plane shear and bending, openings, concentrated loads not implemented. Effective length factor is a user input.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete pad footing (`pad`)

EN 1992-1-1:2004 · Concentric axial load, allowable bearing pressure supplied by geotechnical engineer

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Column load N Ed (ULS) | 1200 | kN |
| Column load N k (SLS) | 870 | kN |
| Allowable bearing pressure | 200 | kPa |
| Column width cx | 300 | mm |
| Column depth cy | 300 | mm |
| Footing thickness h | 500 | mm |
| Cover c (to ground) | 50 | mm |
| Bar φ | 16 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |

**Engine results**

| Result | Value |
|---|---|
| Footing plan size | 2200 × 2200 mm |
| Thickness / depth d | 500 / 426.0 mm |
| Service pressure | 192.3 kPa (allow 200) |
| Net ULS pressure q u | 247.9 kPa |
| Governing moment | 111.9 kNm/m |
| Reinforcement (each way) | T16 @ 300  (670 mm²/m) |
| Footing self-weight | 60.5 kN |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Bearing pressure (SLS) | EN 1997-1 6.5.2 | 192.3 kPa | 200 kPa | 0.96 | PASS |
| Flexural resistance | EN 1992-1-1 6.1 | 111.9 kNm/m | 122.0 kNm/m | 0.92 | PASS |
| Minimum reinforcement | EN 1992-1-1 9.2.1.1(1) | 642 mm²/m | 670 mm²/m | 0.96 | PASS |
| Bar spacing (max) | EN 1992-1-1 9.3.1.1(3) | 300 mm | 400 mm max | 0.75 | PASS |
| Bar spacing (clear) | EN 1992-1-1 8.2(2) | 25 mm min | 284 mm clear | 0.09 | PASS |
| One-way shear | EN 1992-1-1 6.2.2(1) | 129.9 kN/m | 178.7 kN/m | 0.73 | PASS |
| Punching: column face | EN 1992-1-1 6.4.5(3) | 2.347 MPa | 5.280 MPa | 0.44 | PASS |
| Punching: perimeter at 2d | EN 1992-1-1 6.4.4(1) | 0.129 MPa | 0.419 MPa | 0.31 | PASS |

Trace steps: 18; trace hash `2e74d127d6d13ea9184c459a455bb62f8d2734f19f26161c3c83c7c7f0a0434e`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Hand calculation of plan size: B = 2200 mm from (870 + self-weight)/B^2 <= 200 kPa. Flexure, shear and punching not independently reproduced.

**Known limits and open issues:** Eccentric loads, ULS geotechnical bearing, settlement not implemented. Allowable bearing pressure is a user input.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete strip footing (`strip`)

EN 1992-1-1:2004 · Concentric wall line load per metre, allowable bearing pressure supplied by geotechnical engineer

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Wall load N Ed (ULS) | 300 | kN/m |
| Wall load N k (SLS) | 220 | kN/m |
| Allowable bearing pressure | 150 | kPa |
| Wall thickness t | 225 | mm |
| Footing thickness h | 400 | mm |
| Cover c (to ground) | 50 | mm |
| Transverse bar φ | 12 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |

**Engine results**

| Result | Value |
|---|---|
| Footing width B | 1600 mm |
| Thickness / depth d | 400 / 344.0 mm |
| Service pressure | 147.5 kPa (allow 150) |
| Net ULS pressure q u | 187.5 kPa |
| Design moment M Ed | 44.31 kNm/m |
| Transverse steel | T12 @ 200  (565 mm²/m) |
| Longitudinal steel | T8 @ 425 |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Bearing pressure (SLS) | EN 1997-1 6.5.2 | 147.5 kPa | 150 kPa | 0.98 | PASS |
| Flexural resistance | EN 1992-1-1 6.1 | 44.31 kNm/m | 83.1 kNm/m | 0.53 | PASS |
| Minimum reinforcement | EN 1992-1-1 9.2.1.1(1) | 518 mm²/m | 565 mm²/m | 0.92 | PASS |
| Bar spacing (max) | EN 1992-1-1 9.3.1.1(3) | 200 mm | 400 mm max | 0.50 | PASS |
| Bar spacing (clear) | EN 1992-1-1 8.2(2) | 25 mm min | 188 mm clear | 0.13 | PASS |
| One-way shear | EN 1992-1-1 6.2.2(1) | 64.4 kN/m | 154.3 kN/m | 0.42 | PASS |

Trace steps: 17; trace hash `d53e70aa6f25e50beaac07dd3266a44e882956f1d45e9ec23d7fb91709fbe48b`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Hand calculation of width: B = 1600 mm, q_ser = 147.5 kPa for Nk = 220 kN/m, q_allow = 150 kPa. Flexure and shear not independently reproduced.

**Known limits and open issues:** Eccentric loads, ULS geotechnical bearing, settlement, wall stem design not implemented.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete raft foundation (simplified rigid method) (`raft`)

EN 1992-1-1:2004 · One column-line strip, equal symmetric column loads, uniform ground pressure, allowable bearing pressure supplied

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Number of columns in the line | 3 | - |
| Column spacing s | 5000 | mm |
| Overhang from end column to raft edge | 1000 | mm |
| Strip width w (tributary width of the line) | 5000 | mm |
| Load per column N Ed (ULS) | 1500 | kN |
| Load per column N k (SLS) | 1100 | kN |
| Allowable bearing pressure | 150 | kPa |
| Column size c x (along the line) | 400 | mm |
| Column size c y | 400 | mm |
| Raft thickness h | 800 | mm |
| Cover c (top and bottom) | 50 | mm |
| Bar φ | 20 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |

**Engine results**

| Result | Value |
|---|---|
| Raft size L × w | 12.00 m × 5.00 m |
| Service / ULS pressure | 75.0 / 75.0 kPa |
| Max sagging moment (bottom) | 187 kNm on strip (37.4 kNm/m) |
| Max hogging moment (top) | 1500 kNm on strip (300.0 kNm/m) |
| Bottom steel | T20 @ 275 (1142 mm²/m) |
| Top steel | T20 @ 275 (1142 mm²/m) |
| Design shear | 154.5 kN/m |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Bearing pressure (SLS) | EN 1997-1 6.5.2 | 75.0 kPa | 150 kPa | 0.50 | PASS |
| Bottom steel (at columns): flexure | EN 1992-1-1 6.1 | 37.4 kNm/m | 361.4 kNm/m | 0.10 | PASS |
| Bottom steel (at columns): shear | EN 1992-1-1 6.2.2(1) | 154.5 kN/m | 265.8 kN/m | 0.58 | PASS |
| Top steel (between columns): flexure | EN 1992-1-1 6.1 | 300.0 kNm/m | 361.4 kNm/m | 0.83 | PASS |
| Top steel (between columns): shear | EN 1992-1-1 6.2.2(1) | 154.5 kN/m | 265.8 kN/m | 0.58 | PASS |
| Punching, interior column face | EN 1992-1-1 6.4.5(3) | 1.457 MPa | 5.280 MPa | 0.28 | PASS |
| Punching, interior column at 2d | EN 1992-1-1 6.4.4(1) | 0.113 MPa | 0.359 MPa | 0.32 | PASS |

Trace steps: 18; trace hash `5cef9d6e8c987836ec327f80a64649ccc150d63d93d33db57516442f421f810e`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Independent Python statics for 3 columns at 5 m, 1 m overhangs, 5 m strip, 1500 kN each: q_u = 75 kPa, M_sag = 187 kNm, M_hog = 1500 kNm (matches engine).

**Known limits and open issues:** The conventional rigid method is not a code clause and its validity (rigid raft, uniform pressure) is not verified. One direction only. Edge/corner column punching not checked.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Reinforced concrete cantilever retaining wall (`retaining`)

EN 1997-1 / EN 1992-1-1 · Rankine active pressure, horizontal backfill, per metre run

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Stem height H s | 3000 | mm |
| Stem thickness t s (constant) | 350 | mm |
| Base thickness t b | 400 | mm |
| Toe length L t | 800 | mm |
| Total base width B | 3200 | mm |
| Backfill unit weight γ | 18 | kN/m³ |
| Backfill friction angle φ' | 30 | deg |
| Base friction angle δ | 30 | deg |
| Allowable bearing pressure (service) | 200 | kPa |
| Surcharge on backfill q (variable) | 10 | kPa |
| Water height behind wall above base underside (0 = fully drained) | 0 | mm |
| Cover, stem | 40 | mm |
| Cover, base | 50 | mm |
| Stem bar φ | 16 | mm |
| Base bar φ | 16 | mm |
| fck | 30 | MPa |
| fyk | 500 | MPa |

**Engine results**

| Result | Value |
|---|---|
| Wall height / base | 3.40 m / 3.20 m wide |
| K a (characteristic) | 0.3333 |
| Max base pressure | 59.4 kPa (allow 200) |
| Eccentricity e | 0.067 m (B/6 = 0.533) |
| Stem reinforcement | T16 @ 400 (503 mm²/m) |
| Toe reinforcement (bottom) | T16 @ 375 (536 mm²/m) |
| Heel reinforcement (top) | T16 @ 350 (574 mm²/m) |
| Drainage | Assumed fully drained |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Sliding, DA1-1 | EN 1997-1 9.7.4 | 63.8 kN/m | 97.5 kN/m | 0.65 | PASS |
| Sliding, DA1-2 | EN 1997-1 9.7.4 | 60.6 kN/m | 78.0 kN/m | 0.78 | PASS |
| Overturning (EQU) | EN 1997-1 9.7.5 | 88.5 kNm/m | 285.8 kNm/m | 0.31 | PASS |
| Bearing pressure | EN 1997-1 6.5.2 | 59.4 kPa | 200 kPa | 0.30 | PASS |
| Stem: flexure | EN 1992-1-1 6.1 | 59.0 kNm/m | 64.8 kNm/m | 0.91 | PASS |
| Stem: shear | EN 1992-1-1 6.2.2(1) | 51.5 kN/m | 141.4 kN/m | 0.36 | PASS |
| Toe: flexure | EN 1992-1-1 6.1 | 21.3 kNm/m | 78.4 kNm/m | 0.27 | PASS |
| Toe: shear | EN 1992-1-1 6.2.2(1) | 30.6 kN/m | 153.7 kN/m | 0.20 | PASS |
| Heel: flexure | EN 1992-1-1 6.1 | 76.1 kNm/m | 83.9 kNm/m | 0.91 | PASS |
| Heel: shear | EN 1992-1-1 6.2.2(1) | 59.9 kN/m | 153.7 kN/m | 0.39 | PASS |

Trace steps: 26; trace hash `183b1dba3705840bc91d864c615b6921b2b8ec0a791d0cf31a3f87bdd52d889c`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Independent Python calculation of Rankine Ka, sliding DA1-1 = 0.654, DA1-2 = 0.777, overturning (EQU) = 0.310, q_max = 59.4 kPa, stem MEd = 58.95 kNm/m and VEd = 51.45 kN/m for the default wall.

**Known limits and open issues:** Geotechnical partial factors are EN 1997-1 recommended values for DA1; the National Annex may differ. Global stability, settlement, passive resistance not implemented. Drainage is assumed unless water height is entered.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Steel I-beam verification (hot-rolled, UDL) (`steelbeam`)

EN 1993-1-1:2005 · Simply supported, uniformly distributed load, lateral-torsional buckling and deflection

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Span L | 6000 | mm |
| Unrestrained length for LTB L cr | 2000 | mm |
| Load level (1 = top flange, 2 = shear centre / bottom) | 1 | - |
| Section depth h | 300 | mm |
| Flange width b | 150 | mm |
| Web thickness t w | 7.1 | mm |
| Flange thickness t f | 10.7 | mm |
| Root radius r | 15 | mm |
| Yield strength f y | 275 | MPa |
| Permanent load g k (excl. beam self-weight) | 10 | kN/m |
| Variable load q k | 12 | kN/m |
| Deflection limit, span / ratio (variable load) | 360 | - |

**Engine results**

| Result | Value |
|---|---|
| Section class | 1 (flange 1, web 1) |
| A / I y | 5381 mm² / 83.56×10⁶ mm⁴ |
| W pl,y / W el,y | 628.4 / 557.1 ×10³ mm³ |
| M Ed / V Ed | 144.3 kNm / 96.2 kN |
| M c,Rd / M b,Rd | 172.8 / 151.4 kNm |
| M cr / χ LT | 366.8 kNm / 0.876 |
| Deflection | 11.5 mm (limit 16.7) |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Bending resistance | EN 1993-1-1 6.2.5 | 144.3 kNm | 172.8 kNm | 0.84 | PASS |
| Shear resistance | EN 1993-1-1 6.2.6 | 96.2 kN | 407.8 kN | 0.24 | PASS |
| Web shear buckling | EN 1993-1-5 5.1(2) | 39.2 | 55.5 max | 0.71 | PASS |
| Lateral-torsional buckling | EN 1993-1-1 6.3.2 | 144.3 kNm | 151.4 kNm | 0.95 | PASS |
| Deflection | EN 1993-1-1 7.2.1 | 11.5 mm | 16.7 mm | 0.69 | PASS |

Trace steps: 16; trace hash `b923eed42bfb67de150574d929dccee2cf8b192c4c5d3d971d7438eeb79d1846`


**Verification status:** B - checked against published section/standard values (plus author cross-check)

**Cross-checks made so far:** Section properties computed from dimensions agree with published IPE 300 values (A 53.81 cm2, Iy 8356 cm4, Iz 603.8 cm4, Wpl,y 628.4 cm3) within about 1%. Independent Python check of Mcr = 71.0 kNm, chi_LT = 0.402 for L = 6 m, top-flange load.

**Known limits and open issues:** LTB coefficients C1 = 1.127 and C2 = 0.454 come from NCCI SN003b, not the Eurocode text: verify. It_ omits fillet contribution (conservative). Class 4, point loads, axial load not implemented.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Steel I/H column verification (axial compression) (`steelcol`)

EN 1993-1-1:2005 · Hot-rolled doubly symmetric section, flexural buckling about both axes

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Axial compression N Ed (ULS) | 800 | kN |
| Column length L (system length) | 4000 | mm |
| Effective length factor, y-y (strong axis) | 1 | - |
| Effective length factor, z-z (weak axis) | 1 | - |
| Section depth h | 200 | mm |
| Flange width b | 200 | mm |
| Web thickness t w | 9 | mm |
| Flange thickness t f | 15 | mm |
| Root radius r | 18 | mm |
| Yield strength f y | 275 | MPa |

**Engine results**

| Result | Value |
|---|---|
| Section class | 1 (flange 1, web 1) |
| A / I y / I z | 7808 mm² / 56.96 / 20.03 ×10⁶ mm⁴ |
| N c,Rd | 2147 kN |
| λ̄ y / λ̄ z | 0.539 / 0.910 |
| χ y / χ z | 0.866 / 0.594 |
| N b,Rd (governing) | 1275 kN |
| N Ed / N b,Rd | 0.63 |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Cross-section compression | EN 1993-1-1 6.2.4 | 800 kN | 2147 kN | 0.37 | PASS |
| Flexural buckling y-y | EN 1993-1-1 6.3.1.1 | 800 kN | 1860 kN | 0.43 | PASS |
| Flexural buckling z-z | EN 1993-1-1 6.3.1.1 | 800 kN | 1275 kN | 0.63 | PASS |

Trace steps: 12; trace hash `c27aaa56010901133931ab0e5a060bd88f17a4d353b9387491d631eb10a24230`


**Verification status:** B - checked against published section/standard values (plus author cross-check)

**Cross-checks made so far:** Section properties agree with published HEB 200 values (A 78.08 cm2, Iy 5696 cm4, Iz 2003 cm4) within about 1%. Hand calculation of weak-axis buckling: Ncr = 2595 kN, lambda = 0.910, curve c, chi = 0.594, Nb,Rd = 1275 kN.

**Known limits and open issues:** Combined axial and bending, torsional-flexural buckling, class 4 not implemented.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Bolted plate connection in tension (shear and bearing) (`bolted`)

EN 1993-1-8:2005 · Concentric tension, bolts in shear, non-preloaded (Category A), plate net-section and spacing checks

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Design tensile force N Ed (ULS) | 250 | kN |
| Bolt diameter (12, 16, 20, 24, 27 or 30) | 20 | mm |
| Bolt grade (4.6 4.8 5.6 5.8 6.8 8.8 10.9) | 8.8 | - |
| Shear planes per bolt (1 or 2) | 1 | - |
| Shear plane through threads (1) or shank (2) | 1 | - |
| Bolts in a row along the load n 1 | 3 | - |
| Rows across the width n 2 | 2 | - |
| End distance e 1 | 40 | mm |
| Pitch along the load p 1 | 70 | mm |
| Edge distance e 2 | 30 | mm |
| Gauge across the load p 2 | 60 | mm |
| Plate width b | 150 | mm |
| Plate thickness t (thinner outer plate for bearing) | 10 | mm |
| Plate yield f y | 275 | MPa |
| Plate ultimate f u | 430 | MPa |

**Engine results**

| Result | Value |
|---|---|
| Bolts | 6 × M20 grade 8.8 |
| F v,Rd per bolt | 94.0 kN |
| F b,Rd per bolt | 88.3 kN |
| Group shear / bearing | 564.0 / 529.9 kN |
| Plate N pl,Rd / N u,Rd | 412.5 / 328.2 kN |
| Joint length L j | 140 mm (β Lf = 1.000) |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Bolt shear (group) | EN 1993-1-8 Table 3.4 | 250 kN | 564.0 kN | 0.44 | PASS |
| Bolt bearing (group) | EN 1993-1-8 Table 3.4 | 250 kN | 529.9 kN | 0.47 | PASS |
| Plate gross section yielding | EN 1993-1-1 6.2.3(2)a | 250 kN | 412.5 kN | 0.61 | PASS |
| Plate net section | EN 1993-1-1 6.2.3(2)b | 250 kN | 328.2 kN | 0.76 | PASS |
| Minimum edge and spacing | EN 1993-1-8 Table 3.3 | min distances | satisfied | 0.50 | PASS |
| Maximum edge and spacing | EN 1993-1-8 Table 3.3 | max distances | satisfied | 0.50 | PASS |
| Bolts fit within plate width | Geometry | 120 mm | 150 mm | 0.80 | PASS |

Trace steps: 9; trace hash `4928ccc273774aa44ba4f74a199649638e0979264c858afb8d98f264205f4140`


**Verification status:** B - checked against published section/standard values (plus author cross-check)

**Cross-checks made so far:** Tensile stress areas for M12-M30 agree with ISO values (84.3, 157, 245, 353, 459, 561 mm2) within 0.5%. M20 grade 8.8 shear resistance 94.1 kN (standard value). Hand calculation of bearing resistance 88.3 kN per bolt for e1 40, e2 30, p2 60, t 10, fu 430.

**Known limits and open issues:** Block tearing, tension and combined shear-tension, prying, eccentric groups not implemented.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Fillet weld group verification (`weld`)

EN 1993-1-8:2005 · Concentric force on equal-leg fillet welds, directional and simplified methods

**Default inputs**

| Input | Value | Unit |
|---|---|---|
| Total effective weld length L w (sum of all welds) | 300 | mm |
| Throat thickness a | 5 | mm |
| Thinner connected plate t | 10 | mm |
| Force perpendicular to the weld axis F ⊥ | 40 | kN |
| Force parallel to the weld axis F ∥ | 60 | kN |
| Weaker part: yield f y | 275 | MPa |
| Weaker part: ultimate f u | 430 | MPa |

**Engine results**

| Result | Value |
|---|---|
| Throat / length | a = 5 mm, L w = 300 mm |
| Resultant force | 72.1 kN |
| F w,Ed / F w,Rd | 240 / 1168 N/mm |
| Directional stress / limit | 78.9 / 404.7 MPa |
| β w | 0.85 |

**Checks**

| Check | Clause | Demand | Capacity | Utilisation | Result |
|---|---|---|---|---|---|
| Directional method (combined) | EN 1993-1-8 4.5.3.2(4) | 78.9 MPa | 404.7 MPa | 0.19 | PASS |
| Normal stress on throat | EN 1993-1-8 4.5.3.2(4) | 18.9 MPa | 309.6 MPa | 0.06 | PASS |
| Simplified method | EN 1993-1-8 4.5.3.3 | 240 N/mm | 1168 N/mm | 0.21 | PASS |
| Minimum effective length | EN 1993-1-8 4.5.1(1) | 30 mm min | 300 mm | 0.10 | PASS |

Trace steps: 6; trace hash `e6fcf6dc1e50d43d23ce4d3ef7e530699ba79eade67bcae8221bc8c6f02e6395`


**Verification status:** A - author cross-checked (own hand calculation or script; not an independent review)

**Cross-checks made so far:** Hand calculation: f_vw,d = 233.6 MPa and F_w,Rd = 1168 N/mm for a = 5 mm, fu = 430 MPa, beta_w = 0.85.

**Known limits and open issues:** Eccentric weld groups, butt welds, long-joint reduction, fatigue not implemented.

**Sign-off**

| Reviewer (name, registration) | Date | Cases re-derived | Outcome (accepted / accepted with comments / rejected) | Comments |
|---|---|---|---|---|
| | | | | |

## Project dependency engine

Software function (ordering, cycle detection, link substitution, change impact), not an engineering formula. Covered by automated tests: ordering, load passed downstream, multiplier, cycles, unknown links, change impact, blocked nodes. Needs a review of the rule that linked values are design actions chosen by the engineer.

## Backend and security

Database access rules (row-level security), the check/approval workflow, audit hash chain and two-factor rule were tested with simulated users on the live database (outsider isolation, role limits, self-approval blocks, tamper detection). The schema SQL is not yet exported into the repository. A security review by a qualified person is still required before real use.
