use std::{ops::Index, vec};

use crate::{
    frame::{FrameHandle, FrameRenderer}, grid::SpacerSolved, handle::{FallableHandleLike, HandleLike}, BBox
};

use crate::grid::GridSpacer;

use crate::grid::{XName, YName};

use super::SpacerUnit;

#[derive(Clone, Copy, Debug)]
pub enum GridExpandDir {
    X,
    Y,
}

struct HandleSpacerLocation {
    x: usize,
    y: usize,
    handle: FrameHandle,
}

pub struct Grid {
    handles: Vec<HandleSpacerLocation>,
    x_spacer: GridSpacer,
    y_spacer: GridSpacer,
    expand_dir: Option<GridExpandDir>,
    parent_frame_handle: FrameHandle,
    expand_vec: Vec<SpacerSolved>,
    constant_vec: Vec<SpacerSolved>,
}

impl Grid {
    pub fn new(
        parent_frame_handle: FrameHandle,
        x_spacer: GridSpacer,
        y_spacer: GridSpacer,
        expand_dir: Option<GridExpandDir>,
    ) -> Self {
        Self {
            x_spacer,
            y_spacer,
            expand_dir,
            handles: vec![],
            expand_vec: vec![],
            constant_vec: vec![],
            parent_frame_handle,
        }
    }

    pub fn update(&mut self, frames: &mut FrameRenderer) {
        const OUT_OFFSET: usize = 1;
        const IN_OFFSET: usize = 0;
        let bounds = {
            let BBox { x, y, w, h } = frames.get(self.parent_frame_handle).data;
            [x, y, w, h]
        };
        let (outer, inner, bounds_offset) = match self.expand_dir {
            Some(GridExpandDir::X) => (&self.x_spacer, &self.y_spacer, IN_OFFSET),
            Some(GridExpandDir::Y) | None => (&self.y_spacer, &self.x_spacer, OUT_OFFSET),
        };
        let (outer_offset, inner_offset) = (bounds_offset, 1 - bounds_offset);
        
        let (xvec, yvec) = match bounds_offset {
            OUT_OFFSET => (&self.constant_vec, &self.expand_vec),
            IN_OFFSET | _ => (&self.expand_vec, &self.constant_vec),
        };
        self.handles.iter().enumerate().for_each(|(i, handle)| {
            let (x, y) = match outer_offset {
                OUT_OFFSET => (i / yvec.len(), i % yvec.len()),
                _ => (i % xvec.len(), i / xvec.len()),
            };
            let SpacerSolved { pos: x, len: w } = xvec[x];
            let SpacerSolved { pos: y, len: h } = yvec[y];
            frames.update(handle.handle, &BBox { x, y, w, h });
        })
    }
    pub fn is_repeat_x(&self, id: usize) -> bool {
        return matches!(self.x_spacer[id], SpacerUnit::Repeat(_))
    }
    pub fn is_repeat_y(&self, id: usize) -> bool {
        return matches!(self.y_spacer[id], SpacerUnit::Repeat(_))
    }
    fn find_next_slot<'a>(&self, 
        mut candidates: impl std::iter::Iterator<Item = &'a HandleSpacerLocation>, 
        spacers: &[SpacerUnit], 
        which: impl Fn(&HandleSpacerLocation) -> usize
    ) -> Option<usize> {
        if let Some((next_free, _)) = candidates.by_ref().enumerate()
            .find(|(_,h)| 
                !(candidates.any(|h2| 
                    which(h2) == which(h) + 1
                )) || matches!(spacers[which(h)], SpacerUnit::Repeat(_))
            ) {
                Some(next_free)
            } else {
                None
            }
    }
    fn find_next_x(&self) -> usize {
        self.find_next_slot(&self.x_spacer)
    }
    fn find_next_y(&self) -> usize {
        self.find_next_slot(&self.y_spacer)
    }
    pub fn add_frame(&mut self, handle: FrameHandle, x: XName, y: YName) {
        let xi = x.index().unwrap_or_else(||self.);
    }
}
