#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

extern crate proc_macro;

pub(crate) mod component;
pub(crate) mod frame;
pub(crate) mod grid;
pub(crate) mod handle;
pub(crate) mod manager;
pub(crate) mod message;
pub(crate) mod render_actor;
pub(crate) mod units;

pub use component::{ComponentHandle, Interaction, Update};
pub use frame::{FrameData, FrameHandle};
pub use manager::{run, UpdateManager};
pub use units::UserUnits::*;
