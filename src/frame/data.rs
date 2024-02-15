use std::mem;

use bytemuck::{NoUninit, Pod, Zeroable};

use crate::units::VUnit;

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
    pub data: [VUnit; 4],
    pub margin: [VUnit; 4],
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
                    offset: mem::size_of::<VUnit>() as u64 * 4u64,
                    shader_location: 2,
                }
            ]
        }
    }
    pub const BUFFER_INIT_BYTE_COUNT: u64 = 100 * mem::size_of::<Self>() as u64;
}