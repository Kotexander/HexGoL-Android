#![allow(dead_code)]

use std::ops::*;

#[derive(Copy, Clone, PartialEq)]
pub struct HexFract {
    q: f32,
    r: f32,
}
impl HexFract {
    pub const fn new(q: f32, r: f32) -> Self {
        Self { q, r }
    }
    pub const fn q(&self) -> f32 {
        self.q
    }
    pub const fn r(&self) -> f32 {
        self.r
    }
    pub fn s(&self) -> f32 {
        -self.q() - self.r()
    }

    pub fn transform(&self, size: f32) -> [f32; 2] {
        let sqrt_3 = 3.0f32.sqrt();

        let x = size * (3.0 / 2.0 * self.q());
        let y = size * (sqrt_3 / 2.0 * self.q() + sqrt_3 * self.r());

        [x, y]
    }
    pub fn inv_transform(pos: &[f32; 2], size: f32) -> Self {
        let sqrt_3 = 3.0f32.sqrt();

        let q = (2.0 / 3.0 * pos[0]) / size;
        let r = (-1.0 / 3.0 * pos[0] + sqrt_3 / 3.0 * pos[1]) / size;

        Self::new(q, r)
    }

    pub fn round(&self) -> Self {
        let mut q = self.q().round();
        let mut r = self.r().round();
        let s = self.s().round();

        let q_diff = (q - self.q()).abs();
        let r_diff = (r - self.r()).abs();
        let s_diff = (s - self.s()).abs();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        }

        Self::new(q, r)
    }
}
impl From<HexInt> for HexFract {
    fn from(hex: HexInt) -> Self {
        Self::new(hex.q() as f32, hex.r() as f32)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct HexInt {
    q: i32,
    r: i32,
}
impl HexInt {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
    pub const fn q(&self) -> i32 {
        self.q
    }
    pub const fn r(&self) -> i32 {
        self.r
    }
    pub const fn s(&self) -> i32 {
        -self.q() - self.r()
    }
}
impl Add for HexInt {
    type Output = HexInt;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.q + rhs.q, self.r + rhs.r)
    }
}
impl From<HexFract> for HexInt {
    fn from(hex: HexFract) -> Self {
        Self::new(hex.q() as i32, hex.r() as i32)
    }
}
