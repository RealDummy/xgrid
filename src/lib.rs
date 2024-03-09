#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

extern crate proc_macro;

pub(crate) mod component;
pub(crate) mod events;
pub(crate) mod frame;
pub(crate) mod grid;
pub(crate) mod handle;
pub(crate) mod manager;
pub(crate) mod observer;
pub(crate) mod render_actor;
pub(crate) mod units;
pub(crate) mod update_queue;

pub use component::{Builder, Component, SystemEvents, UpdateQueue};
pub use component::{Interaction, State};
pub use events::{ButtonState, KeyboardEvent, KeyboardKey, MouseButton, MouseEvent};
pub use frame::{FrameData, FrameHandle};
pub use manager::{run, RenderManager, Rect};
pub use observer::{EventDispatcher, Subscriber};
pub use render_actor::FrameMessage;
pub use units::UserUnits::*;
pub use update_queue::back::UpdateMsg;
