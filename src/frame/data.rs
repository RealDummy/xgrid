use std::mem;

use bytemuck::{NoUninit, Pod, Zeroable};
use wgpu::naga::Handle;

use crate::{units::VUnit, BBox, MarginBox};

#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Loc {
    X,
    Y,
    W,
    H,
}

pub enum Border {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,
}

#[derive(Pod, Clone, Copy, Zeroable, Debug)]
#[repr(C)]
pub struct FrameData {
    pub data: BBox,
    pub margin: MarginBox,
    pub color: [u8; 4],
    pub camera_index: u32,
}

impl FrameData {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32x4,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32x4,
                    offset: mem::size_of::<BBox>() as u64,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Unorm8x4,
                    offset: mem::size_of::<(BBox, MarginBox)>() as u64,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: mem::size_of::<(BBox, MarginBox, [u8; 4])>() as u64,
                    shader_location: 4,
                }
            ]
        }
    }
    pub const BUFFER_INIT_BYTE_COUNT: u64 = 100 * mem::size_of::<Self>() as u64;
}

pub trait FrameSizer {
    fn size_next(&self);
}