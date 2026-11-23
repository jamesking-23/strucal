//! Solid timber property data transcribed from the supplied source: Table 11.3 (= Table 1 of EN 338), Table 11.4, 11.5, 11.9 of the
//! book chapter "Eurocode 5: Design of timber structures" (values for the UK National Annex as printed there).
//! The D70 value of E0,05 is printed as 6.8, which is inconsistent with the other classes (D60 = 14.3): treated as unavailable.

pub struct Class { pub name: &'static str, pub fmk: f64, pub ft0k: f64, pub ft90k: f64, pub fc0k: f64, pub fc90k: f64, pub fvk: f64,
    pub e0mean: f64, pub e005: Option<f64>, pub e90mean: f64, pub gmean: f64, pub rhok: f64, pub rhomean: f64 }

macro_rules! c { ($n:expr, $fm:expr, $ft0:expr, $ft90:expr, $fc0:expr, $fc90:expr, $fv:expr, $e0:expr, $e005:expr, $e90:expr, $g:expr, $rk:expr, $rm:expr) => {
    Class { name: $n, fmk: $fm, ft0k: $ft0, ft90k: $ft90, fc0k: $fc0, fc90k: $fc90, fvk: $fv, e0mean: $e0 * 1000.0, e005: $e005, e90mean: $e90 * 1000.0, gmean: $g * 1000.0, rhok: $rk, rhomean: $rm } } }

pub const CLASSES: [Class; 15] = [
    c!("C14", 14.0, 8.0, 0.3, 16.0, 4.3, 1.7, 7.0, Some(4.7 * 1000.0), 0.23, 0.44, 290.0, 350.0),
    c!("C16", 16.0, 10.0, 0.3, 17.0, 4.6, 1.8, 8.0, Some(5.4 * 1000.0), 0.27, 0.50, 310.0, 370.0),
    c!("C18", 18.0, 11.0, 0.3, 18.0, 4.8, 2.0, 9.0, Some(6.0 * 1000.0), 0.30, 0.56, 320.0, 380.0),
    c!("C22", 22.0, 13.0, 0.3, 20.0, 5.1, 2.4, 10.0, Some(6.7 * 1000.0), 0.33, 0.63, 340.0, 410.0),
    c!("C24", 24.0, 14.0, 0.4, 21.0, 5.3, 2.5, 11.0, Some(7.4 * 1000.0), 0.37, 0.69, 350.0, 420.0),
    c!("C27", 27.0, 16.0, 0.4, 22.0, 5.6, 2.8, 12.0, Some(8.0 * 1000.0), 0.40, 0.75, 370.0, 450.0),
    c!("C30", 30.0, 18.0, 0.4, 23.0, 5.7, 3.0, 12.0, Some(8.0 * 1000.0), 0.40, 0.75, 380.0, 460.0),
    c!("C35", 35.0, 21.0, 0.4, 25.0, 6.0, 3.4, 13.0, Some(8.7 * 1000.0), 0.43, 0.81, 400.0, 480.0),
    c!("C40", 40.0, 24.0, 0.4, 26.0, 6.3, 3.8, 14.0, Some(9.4 * 1000.0), 0.47, 0.88, 420.0, 500.0),
    c!("D30", 30.0, 18.0, 0.6, 23.0, 8.0, 3.0, 10.0, Some(8.0 * 1000.0), 0.64, 0.60, 530.0, 640.0),
    c!("D35", 35.0, 21.0, 0.6, 25.0, 8.4, 3.4, 10.0, Some(8.7 * 1000.0), 0.69, 0.65, 560.0, 670.0),
    c!("D40", 40.0, 24.0, 0.6, 26.0, 8.8, 3.8, 11.0, Some(9.4 * 1000.0), 0.75, 0.70, 590.0, 700.0),
    c!("D50", 50.0, 30.0, 0.6, 29.0, 9.7, 4.6, 14.0, Some(11.8 * 1000.0), 0.93, 0.88, 650.0, 780.0),
    c!("D60", 60.0, 36.0, 0.7, 32.0, 10.5, 5.3, 17.0, Some(14.3 * 1000.0), 1.13, 1.06, 700.0, 840.0),
    c!("D70", 70.0, 42.0, 0.9, 34.0, 13.5, 6.0, 20.0, None, 1.33, 1.25, 900.0, 1080.0),
];

pub fn class_options() -> Vec<(f64, &'static str)> { CLASSES.iter().enumerate().map(|(i, c)| ((i + 1) as f64, c.name)).collect() }
pub fn class_of(v: f64) -> Option<&'static Class> { let i = v.round() as i64; if (1..=15).contains(&i) { Some(&CLASSES[(i - 1) as usize]) } else { None } }
pub fn service_options() -> Vec<(f64, &'static str)> { vec![(1.0, "1 (heated interior, MC up to 12%)"), (2.0, "2 (unheated interior, MC up to 20%)"), (3.0, "3 (exterior, MC above 20%)")] }
pub fn duration_options() -> Vec<(f64, &'static str)> { vec![(1.0, "Permanent (> 10 years)"), (2.0, "Long-term (6 months - 10 years)"), (3.0, "Medium-term (1 week - 6 months)"), (4.0, "Short-term (< 1 week)"), (5.0, "Instantaneous")] }
/// Table 11.5: kmod for solid timber by service class and load-duration class.
pub fn kmod(service: u8, duration: u8) -> f64 {
    let t12 = [0.60, 0.70, 0.80, 0.90, 1.10]; let t3 = [0.50, 0.55, 0.65, 0.70, 0.90];
    let i = (duration.clamp(1, 5) - 1) as usize; if service == 3 { t3[i] } else { t12[i] }
}
/// Table 11.9: kdef for solid timber.
pub fn kdef(service: u8) -> f64 { match service { 1 => 0.60, 2 => 0.80, _ => 2.00 } }
/// Table 11.4: partial factor for solid timber, fundamental combinations.
pub const GAMMA_M: f64 = 1.3;
/// Equation 11.2: depth factor for solid timber below 150 mm.
pub fn kh(depth_mm: f64) -> f64 { if depth_mm >= 150.0 { 1.0 } else { (150.0 / depth_mm).powf(0.2).min(1.3) } }
/// Equations 11.19-11.21.
pub fn kcrit(lam_rel_m: f64) -> f64 { if lam_rel_m <= 0.75 { 1.0 } else if lam_rel_m <= 1.4 { 1.56 - 0.75 * lam_rel_m } else { 1.0 / (lam_rel_m * lam_rel_m) } }
