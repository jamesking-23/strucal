//! Dimensionally-typed quantities. Base units: N and mm.
use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dim { pub n: i8, pub mm: i8 }

pub const SCALAR: Dim = Dim { n: 0, mm: 0 };
pub const LENGTH: Dim = Dim { n: 0, mm: 1 };
pub const AREA: Dim = Dim { n: 0, mm: 2 };
pub const FORCE: Dim = Dim { n: 1, mm: 0 };
pub const MOMENT: Dim = Dim { n: 1, mm: 1 };
pub const STRESS: Dim = Dim { n: 1, mm: -2 };

#[derive(Clone, Copy, Debug)]
pub struct Q { pub v: f64, pub d: Dim }

impl Q {
    pub fn new(v: f64, d: Dim) -> Q { Q { v, d } }
    pub fn mm(v: f64) -> Q { Q::new(v, LENGTH) }
    pub fn mm2(v: f64) -> Q { Q::new(v, AREA) }
    pub fn n(v: f64) -> Q { Q::new(v, FORCE) }
    pub fn mpa(v: f64) -> Q { Q::new(v, STRESS) }
    pub fn knm(v: f64) -> Q { Q::new(v * 1e6, MOMENT) }
    pub fn kn(v: f64) -> Q { Q::new(v * 1e3, FORCE) }
    pub fn scalar(v: f64) -> Q { Q::new(v, SCALAR) }
    /// Returns the value only if the dimension matches; otherwise an error.
    pub fn expect(self, d: Dim) -> Result<f64, String> {
        if self.d == d { Ok(self.v) } else { Err(format!("dimension mismatch: have {:?}, need {:?}", self.d, d)) }
    }
    pub fn sqrt(self) -> Result<Q, String> {
        if self.d.n % 2 != 0 || self.d.mm % 2 != 0 { return Err("sqrt of odd dimension".into()); }
        Ok(Q::new(self.v.sqrt(), Dim { n: self.d.n / 2, mm: self.d.mm / 2 }))
    }
}
impl Add for Q { type Output = Q; fn add(self, o: Q) -> Q { assert_eq!(self.d, o.d, "cannot add unlike quantities"); Q::new(self.v + o.v, self.d) } }
impl Sub for Q { type Output = Q; fn sub(self, o: Q) -> Q { assert_eq!(self.d, o.d, "cannot subtract unlike quantities"); Q::new(self.v - o.v, self.d) } }
impl Mul for Q { type Output = Q; fn mul(self, o: Q) -> Q { Q::new(self.v * o.v, Dim { n: self.d.n + o.d.n, mm: self.d.mm + o.d.mm }) } }
impl Div for Q { type Output = Q; fn div(self, o: Q) -> Q { Q::new(self.v / o.v, Dim { n: self.d.n - o.d.n, mm: self.d.mm - o.d.mm }) } }
impl Mul<f64> for Q { type Output = Q; fn mul(self, k: f64) -> Q { Q::new(self.v * k, self.d) } }
impl Div<f64> for Q { type Output = Q; fn div(self, k: f64) -> Q { Q::new(self.v / k, self.d) } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn stress_times_area_is_force() { assert_eq!((Q::mpa(10.0) * Q::mm2(5.0)).d, FORCE); }
    #[test] fn force_times_length_is_moment() { assert_eq!((Q::n(2.0) * Q::mm(3.0)).d, MOMENT); }
    #[test] #[should_panic] fn force_plus_moment_rejected() { let _ = Q::n(1.0) + Q::knm(1.0); }
    #[test] fn expect_rejects_wrong_dim() { assert!(Q::mm(1.0).expect(AREA).is_err()); }
}
