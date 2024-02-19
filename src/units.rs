use std::{fmt, io::{BufWriter, Read}, ops::{Add, AddAssign, Div, Mul, Sub}};

use bytemuck::{Pod, Zeroable};


pub type Pixelt = i32;
pub type Ratiot = f32;
pub type Fractiont = u32;

#[derive(Debug, Clone, Copy)]
pub enum UserUnits {
    Zero,
    Pixel (Pixelt),
    Ratio (Ratiot),
    Fraction (Fractiont),
}

/// i32 but 6 bits are for sub VUnit precision the max value is
/// `i32::MAX >> 6` or 33_554_431 and 63/64
#[derive(Pod, Clone, Copy, Zeroable)]
#[repr(C)]
pub struct VUnit(i32);

impl VUnit {
    const PRECISION_BITS: i32 = 6;
    pub fn pix(&self) -> f32 {
        let full_pix = (self.0 / (1<< Self::PRECISION_BITS )) as f32;
        let sub_pix = (self.0 % (1<< Self::PRECISION_BITS )) as f32;

        return full_pix + sub_pix;

    }
    pub fn new(p: i32) -> VUnit {
        Self (
            p.into()
        )
    }
}

impl From<i32> for VUnit {
    fn from(value: i32) -> Self {
        VUnit(value << Self::PRECISION_BITS)
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
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        let precision = 2.0f64.powi(-VUnit::PRECISION_BITS);
        let value = precision * self.0 as f64;
        formatter.write_fmt(format_args!("{:.2}", value))
    }
}


#[cfg(test)]
mod test {
    use super::*;
    

}