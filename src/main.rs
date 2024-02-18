

use std::{hash::{Hash, Hasher}, marker::PhantomData};

use xgrid::run;


fn main() {
    pollster::block_on(run())
}
