

use std::{hash::{Hash, Hasher}, marker::PhantomData};

use wgpu_ui::run;

fn main() {
    pollster::block_on(run())
}
