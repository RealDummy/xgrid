use std::{fmt, io::{BufWriter, Read}};

use bytemuck::{Pod, Zeroable};


pub type Pxt = f32;
pub type Rlt = f32;
pub type Frt = f32;

pub enum UserUnits {
    Zero,
    Pixel (Pxt),
    Rel (Rlt),
    Frac (Frt),
}

/// i32 but 6 bits are for sub VUnit precision the max value is
/// `i32::MAX >> 6` or 33_554_431 and 63/64
#[derive(Pod, Clone, Copy, Zeroable)]
#[repr(C)]
pub struct VUnit(pub i32);

impl VUnit {
    const PRECISION_BITS: i32 = 6;
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