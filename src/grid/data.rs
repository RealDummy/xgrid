use std::{
    iter::{self},
    ops::Index,
    vec,
};

use log::debug;

use crate::{
    frame::{FrameHandle, FrameRenderer},
    handle::{FallableHandleLike, HandleLike},
    manager::BBox,
    units::{Fractiont, UserUnits, VUnit},
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
    major: usize,
    cross: usize,
    handle: FrameHandle,
}

pub struct GridData {
    handles: Vec<HandleSpacerLocation>,
    cross_spacer: GridSpacer,
    major_spacer: GridSpacer,
    expand_dir: Option<GridExpandDir>,
    parent_frame_handle: FrameHandle,
    major_row_counts: Vec<usize>,
}

#[derive(Clone)]
pub struct SpacerSolved {
    pub pos: VUnit,
    pub len: VUnit,
    pub count: usize,
}

#[derive(Clone)]
enum SolveUnits {
    Exact(VUnit),
    Fraction(Fractiont),
}

fn units_solve(u: UserUnits, len: VUnit) -> SolveUnits {
    use SolveUnits::*;
    match u {
        UserUnits::Pixel(p) => Exact(p.into()),
        UserUnits::Ratio(f) => Exact(((len.pix() * f).round() as i32).into()),
        UserUnits::Zero => Exact(0.into()),
        UserUnits::Fraction(f) => Fraction(f),
    }
}

fn solve_spacer<'a>(
    items: impl Iterator<Item = &'a HandleSpacerLocation> + Clone + 'a,
    spacer_template: &'a GridSpacer,
    which: impl Fn(&HandleSpacerLocation) -> usize + Clone + 'a,
    pos: VUnit,
    len: VUnit,
) -> impl Iterator<Item = SpacerSolved> + 'a {
    let iter_res = spacer_template
        .iter()
        .enumerate()
        .flat_map(move |(i, u)| {
            let count = items.clone().filter(|s| which(s) == i).count();
            iter::repeat((count, u)).take(count.max(1))
        })
        .map(|(i, s)| {
            (
                i,
                match s {
                    SpacerUnit::Unit(u) => u,
                    SpacerUnit::Repeat(u) => u,
                },
            )
        })
        .map(move |(i, u)| (i, units_solve(*u, len)));
    //debug!("count b4 {}", iter_res.clone().count());
    let (total_f, taken_u) = iter_res
        .clone()
        .fold((0, 0.into()), |(a, rest), (_i, u)| match u {
            SolveUnits::Fraction(f) => (a + f, rest),
            SolveUnits::Exact(v) => (a, rest + v),
        });
    let units_remaining = (len - taken_u).max(0.into());
    let mut curr_pos = pos;
    let iter_res = iter_res.map(move |(i, u)| SpacerSolved {
        count: i,
        pos: curr_pos,
        len: match u {
            SolveUnits::Exact(u) => {
                curr_pos += u;
                u
            }
            SolveUnits::Fraction(f) => {
                let u = ((f as i32) * units_remaining) / (total_f as i32);
                curr_pos += u;
                u
            }
        },
    });
    //debug!("count: {}", iter_res.clone().count());
    iter_res
}

impl GridData {
    pub fn new(
        parent_frame_handle: FrameHandle,
        x_spacer: GridSpacer,
        y_spacer: GridSpacer,
        expand_dir: Option<GridExpandDir>,
    ) -> Self {
        let (major_spacer, cross_spacer) = match expand_dir {
            Some(GridExpandDir::X) => (x_spacer, y_spacer),
            _ => (y_spacer, x_spacer),
        };
        Self {
            major_spacer,
            cross_spacer,
            expand_dir,
            handles: vec![],
            major_row_counts: vec![],
            parent_frame_handle,
        }
    }
    pub fn parent(&self) -> FrameHandle {
        return self.parent_frame_handle;
    }
    pub fn update(&mut self, frames: &mut FrameRenderer) {
        let parent_box = frames.get(self.parent_frame_handle).data;
        debug!("grid {} box {:?}",self.parent_frame_handle.index(), parent_box);
        self.handles.sort_by_key(|h| (h.major, h.cross));
        let BBox {
            x: major_pos,
            y: cross_pos,
            w: major_len,
            h: cross_len,
        } = match self.expand_dir {
            Some(GridExpandDir::X) => parent_box,
            _ => BBox {
                x: parent_box.y,
                y: parent_box.x,
                w: parent_box.h,
                h: parent_box.w,
            },
        };
        let cross_solve: Vec<_> = solve_spacer(
            self.handles.iter().take(0),
            &self.cross_spacer,
            |h| h.cross,
            cross_pos,
            cross_len,
        )
        .collect();

        for cross_index in 0..self.cross_spacer.len() {
            //debug!("major_index: {}", major_index);
            let mut major_iter = self.handles.iter().filter(|h| h.cross == cross_index);
            let major_solve = solve_spacer(
                major_iter.clone(),
                &self.major_spacer,
                |h| h.major,
                major_pos,
                major_len,
            );
            major_solve.for_each(|solve| {
                let cross_solve = &cross_solve[cross_index];
                let bounds = match self.expand_dir {
                    Some(GridExpandDir::X) => BBox {
                        x: solve.pos,
                        y: cross_solve.pos,
                        w: solve.len,
                        h: cross_solve.len,
                    },
                    _ => BBox {
                        y: solve.pos,
                        x: cross_solve.pos,
                        h: solve.len,
                        w: cross_solve.len,
                    },
                };
                //debug!("cross index: {}", cross_index);
                major_iter
                    .by_ref()
                    .take(solve.count.min(1))
                    .for_each(|loc| {
                        debug!("frame: {} {:?}",loc.handle.index(), bounds);
                        //debug!("frame {}: {:?} {}", loc.handle.index(), bounds, solve.count);
                        frames.update(loc.handle, &bounds);
                    })
            })
        }
    }

    fn find_next_slot<'a, T>(
        &self,
        competitors: T,
        slot_spacers: &[SpacerUnit],
        which: impl Fn(&HandleSpacerLocation) -> usize,
    ) -> Option<usize>
    where
        T: std::iter::Iterator<Item = &'a HandleSpacerLocation> + Clone,
    {
        if let Some((next_free, _)) = slot_spacers
            .iter()
            .enumerate()
            .find(|(i, _h)| !(competitors.clone().any(|h2| which(h2) == *i)))
        {
            Some(next_free)
        } else {
            slot_spacers
                .iter()
                .enumerate()
                .map(|(i, _)| (competitors.clone().filter(|h| which(h) == i).count(), i))
                .min()
                .map(|t| t.1)
        }
    }
    fn find_next_major_spacer(&self, cross_index: Option<usize>) -> Option<usize> {
        let candidates = self.handles.iter().filter(|h| match cross_index {
            Some(ci) => ci == h.cross,
            None => true,
        });
        self.find_next_slot(candidates, &self.major_spacer.as_slice(), |h| h.major)
    }
    fn find_next_cross_spacer(&self, major_index: Option<usize>) -> Option<usize> {
        let candidates = self.handles.iter().filter(|h| match major_index {
            Some(mi) => mi == h.major,
            None => true,
        });
        self.find_next_slot(candidates, &self.cross_spacer.as_slice(), |h| h.cross)
    }
    pub fn add_frame(&mut self, handle: FrameHandle, x: XName, y: YName) -> Result<(), ()> {
        let (major_index, cross_index) = match self.expand_dir {
            Some(GridExpandDir::X) => (x.index(), y.index()),
            _ => (y.index(), x.index()),
        };
        let next_major_index = match major_index {
            None => {
                if let Some(xi) = self.find_next_major_spacer(cross_index) {
                    xi
                } else {
                    return Err(());
                }
            }
            Some(n) => n,
        };
        //debug!("{next_major_index} ? {:?}", handle.index());
        let next_cross_index = match cross_index {
            None => {
                if let Some(yi) = self.find_next_cross_spacer(major_index) {
                    yi
                } else {
                    return Err(());
                }
            }
            Some(n) => n,
        };
        //debug!("{next_major_index} {next_cross_index} {:?}", handle.index());
        self.handles.push(HandleSpacerLocation {
            major: next_major_index,
            cross: next_cross_index,
            handle,
        });
        return Ok(());
    }
}
