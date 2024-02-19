use std::{iter, marker::PhantomData, vec};


use crate::{frame::{FrameHandle, FrameRenderer}, grid::SpacerSolved, handle::Handle, manager::UpdateManager, units::{Fractiont, UserUnits, VUnit}, BBox, WorldView};

use crate::grid::{GridHandle, GridSpacer};

use super::{XName, YName};

#[derive(Clone, Copy, Debug)]
pub enum GridExpandDir {
    X,
    Y,
}

pub struct Grid {
    pub handles: Vec<Option<FrameHandle>>,
    pub x_spacer: GridSpacer,
    pub y_spacer: GridSpacer,
    pub expand_dir: GridExpandDir,
    pub parent_frame_handle: FrameHandle,
    outer_vec: Vec<SpacerSolved>,
    inner_vec: Vec<SpacerSolved>,
}   

impl Grid {
    pub fn new(parent_frame_handle: FrameHandle, x_spacer: GridSpacer, y_spacer: GridSpacer, expand_dir: Option<GridExpandDir> ) -> Self {
        Self {
            x_spacer,
            y_spacer,
            handles: vec![],
            expand_dir: expand_dir.unwrap_or(GridExpandDir::Y),
            outer_vec: vec![],
            inner_vec: vec![],
            parent_frame_handle,
        }
    }
    
    pub fn update(&mut self, frames: &mut FrameRenderer) {
        const OUT_OFFSET: usize = 1;
        const IN_OFFSET: usize = 0;
        let bounds = {
            let BBox { x, y, w, h } = frames.get(self.parent_frame_handle).data;
            [x,y,w,h]
        };
        let (outer, inner, bounds_offset) =  match self.expand_dir {
            GridExpandDir::X => (&self.x_spacer, &self.y_spacer, IN_OFFSET),
            GridExpandDir::Y => (&self.y_spacer, &self.x_spacer, OUT_OFFSET),
        };
        let (outer_offset, inner_offset) = (bounds_offset, 1-bounds_offset);
        let repeat_count = (self.handles.len() / inner.len()).checked_sub(outer.len() - 1).unwrap_or(0);
        let outer_spacer = super::renderer::expand_spacer(outer, bounds[outer_offset], bounds[2 + outer_offset], repeat_count);
        let inner_spacer = super::renderer::expand_spacer(inner, bounds[inner_offset], bounds[2 + inner_offset], repeat_count);
        self.outer_vec.clear();
        self.inner_vec.clear();
        self.outer_vec.extend(outer_spacer);
        self.inner_vec.extend(inner_spacer);
        let (xvec, yvec) = match bounds_offset {
            OUT_OFFSET => (&self.inner_vec,&self.outer_vec),
            IN_OFFSET | _ => (&self.outer_vec,&self.inner_vec), 
        };
        self.handles.iter().enumerate().for_each(|(i, h)|{
            let Some(handle) = h else {
                return;
            };
            let (x,y) = match outer_offset {
                OUT_OFFSET => (i/yvec.len(), i%yvec.len()),
                _ => (i%xvec.len(), i/xvec.len())
            };
            let SpacerSolved {
                pos: x,
                len: w,
            } = xvec[x];
            let SpacerSolved {
                pos: y,
                len: h,
            } = yvec[y];
            frames.update(*handle, &BBox{x,y,w,h});

        })
    }
    pub fn add_frame(&mut self, handle: FrameHandle, x: XName, y: YName) {
        self.handles.push(Some(handle))
    }

} 
