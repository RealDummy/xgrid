#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

extern crate proc_macro;

pub(crate) mod component;
pub(crate) mod frame;
pub(crate) mod grid;
pub(crate) mod handle;
pub(crate) mod manager;
pub(crate) mod message;
pub(crate) mod units;

pub use component::{Update, UpdateComponent, Interaction, ComponentHandle};
pub use manager::{run, UpdateManager};
pub use units::UserUnits::*;
pub use frame::{FrameHandle, FrameData};
