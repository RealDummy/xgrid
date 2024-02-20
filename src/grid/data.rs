use std::{ops::Index, vec};

use crate::{
    frame::{FrameHandle, FrameRenderer}, grid::SpacerSolved, handle::{FallableHandleLike, HandleLike}, BBox
};

use crate::grid::GridSpacer;

use super::{SpacerUnit, XName, YName};

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
    pub fn add_frame(&mut self, handle: FrameHandle, x: XName, y: YName) {
        match (x.index(), y.index()) {
            (Some(xi), Some(yi)) => {
                let new = HandleSpacerLocation {
                    x: xi,
                    y: yi,
                    handle
                };
                let spot = self.handles.iter_mut()
                .take_while(|h| !self.is_repeat_x(h.x) && !self.is_repeat_y(h.y))
                .find(|h| h.x == xi && h.y == yi);
                match spot {
                    Some(r) => {*r = new;},
                    None => {self.handles.push(new)}
                };
            }
            (None, Some(yi)) => {
                let mut replace = self.handles.iter_mut()
                    .filter(|h| h.y == yi)
                    .take_while(|h| !self.is_repeat_x(h.x))
                    .find(|h| !self.handles.iter().any(|h2| h2.x == h.x + 1));
                match replace {
                    Some(h) => {h.handle = handle}, // case handles iter finds missing spot before x repeats
                    None if self.is_repeat_y(yi) => { // cant find missing spot because x has no repeats
                        let xi = self.handles.iter().max_by_key(|h| h.x).map_or(0, |h| h.x);
                        self.handles.push(HandleSpacerLocation {
                            x: xi,
                            y: yi,
                            handle
                        });
                    }
                    None => { //case 
                        let xi = if let Some(GridExpandDir::X) = self.expand_dir {
                            self.handles.iter().find(|h| self.is_repeat_x(h.x)).map_or(0, |h| h.x)
                        } else {
                            self.handles.iter().max_by_key(|h| h.x).map_or(0, |h| h.x)
                        };
                        self.handles.push(HandleSpacerLocation {
                            x: xi,
                            y: yi,
                            handle
                        });

                    }
                }
                
            }
            (Some(xi), None) => {
                let yi = self.handles.iter()
                .fold(0, |a,h| usize::max(h.y, a));
                self.handles.push(HandleSpacerLocation {
                    x: xi,
                    y: yi,
                    handle
                });
            }
            (None, None) => {}
        }


        let (exp_template, const_template) = match &self.expand_dir {
            Some(GridExpandDir::X) => (&self.x_spacer, &self.y_spacer),
            Some(GridExpandDir::Y) | None => (&self.y_spacer, &self.x_spacer),
        };
        let fixed_expand_len = exp_template.iter().filter(|&&s| matches!(s, SpacerUnit::Unit(_))).count(); //slow?
        let fixed_constant_len = const_template.len();
        let repeat_count = (self.handles.len() / self.constant_vec.len())
            .checked_sub(self.expand_vec.len() - 1)
            .unwrap_or(0);
        
        self.expand_vec.extend(outer_spacer);
        self.constant_vec.extend(inner_spacer);
        let xi = x.index();
        let yi = y.index();
    }
}
