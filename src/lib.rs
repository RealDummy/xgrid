#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

extern crate proc_macro;

pub(crate) mod component;
pub(crate) mod frame;
pub(crate) mod grid;
pub(crate) mod handle;
pub(crate) mod manager;
pub(crate) mod render_actor;
pub(crate) mod units;
pub(crate) mod update_queue;

pub use component::{Builder, Component, UpdateQueue};
pub use component::{Interaction, State};
pub use frame::{FrameData, FrameHandle};
pub use manager::{run, RenderManager};
pub use render_actor::FrameMessage;
pub use units::UserUnits::*;
pub use update_queue::back::UpdateMsg;
