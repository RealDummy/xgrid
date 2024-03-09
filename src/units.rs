use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use bytemuck::{Pod, Zeroable};

pub type Pixelt = i32;
pub type Ratiot = f32;
pub type Fractiont = u32;

#[derive(Debug, Clone, Copy)]
pub enum UserUnits {
    Zero,
    Pixel(Pixelt),
    Ratio(Ratiot),
    Fraction(Fractiont),
}

/// i32 but 6 bits are for sub VUnit precision the max value is
/// `i32::MAX >> 6` or 33_554_431 and 63/64
#[derive(Pod, Clone, Copy, Zeroable)]
#[repr(C)]
pub struct VUnit(i32);

impl VUnit {
    const PRECISION_BITS: i32 = 6;
    pub fn pix(&self) -> f32 {
        let full_pix = (self.0 >> Self::PRECISION_BITS) as f32;
        let sub_pix =
            (self.0 % (1 << Self::PRECISION_BITS)) as f32 / 2.0f32.powi(Self::PRECISION_BITS);

        return full_pix + sub_pix;
    }
    pub fn new<T: Into<VUnit>>(p: T) -> VUnit {
        p.into()
    }
    pub(crate) fn truct_to_whole(&self) -> i32 {
        self.0 >> Self::PRECISION_BITS
    }
}

impl PartialEq for VUnit {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
impl Eq for VUnit {}

impl PartialOrd for VUnit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(i32::cmp(&self.0, &other.0))
    }
}

impl Ord for VUnit {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        i32::cmp(&self.0, &other.0)
    }
}

impl From<i32> for VUnit {
    fn from(value: i32) -> Self {
        VUnit(value << Self::PRECISION_BITS)
    }
}

impl From<u32> for VUnit {
    fn from(value: u32) -> Self {
        VUnit((value as i32) << Self::PRECISION_BITS)
    }
}

impl Add for VUnit {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        VUnit(self.0 + rhs.0)
    }
}

impl AddAssign for VUnit {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for VUnit {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        VUnit(self.0 - rhs.0)
    }
}

impl Mul<i32> for VUnit {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        VUnit(self.0 * rhs)
    }
}

impl Mul<VUnit> for i32 {
    type Output = VUnit;
    fn mul(self, rhs: VUnit) -> Self::Output {
        VUnit(self * rhs.0)
    }
}

impl Div<i32> for VUnit {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        VUnit(self.0 / rhs)
    }
}

impl Div for VUnit {
    type Output = f32;

    fn div(self, rhs: Self) -> Self::Output {
        self.pix() / rhs.pix()
    }
}
impl fmt::Debug for VUnit {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        formatter.write_fmt(format_args!("{:.2}", self.pix()))
    }
}

#[cfg(test)]
mod test {}
