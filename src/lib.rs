
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub(crate) mod frame;
pub(crate) mod grid;
pub(crate) mod handle;
pub(crate) mod manager;
pub(crate) mod units;
pub(crate) mod message;
pub(crate) mod component;

pub use manager::run;
    
