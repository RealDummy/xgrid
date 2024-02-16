use std::mem;

use bytemuck::{NoUninit, Pod, Zeroable};

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

#[derive(Pod, Clone, Copy, Zeroable)]
#[repr(C)]
pub struct FrameData {
    pub data: BBox,
    pub margin: MarginBox,
    pub color: [u8; 4],
    pub camera_index: u16,
    pub _pad1: u16,
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
                    offset: mem::size_of::<[VUnit; 4]>() as u64,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Unorm8x4,
                    offset: mem::size_of::<([VUnit; 4], [VUnit; 4])>() as u64,
                    shader_location: 3,
                }
            ]
        }
    }
    pub const BUFFER_INIT_BYTE_COUNT: u64 = 100 * mem::size_of::<Self>() as u64;
}