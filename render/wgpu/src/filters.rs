mod blur;
mod color_matrix;
mod drop_shadow;
mod glow;
mod shader;

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::blur::BlurFilter;
use crate::filters::color_matrix::ColorMatrixFilter;
use crate::filters::drop_shadow::DropShadowFilter;
use crate::filters::glow::GlowFilter;
use crate::filters::shader::ShaderFilter;
use crate::surface::target::CommandTarget;
use bytemuck::{Pod, Zeroable};
use ruffle_render::filters::Filter;
use swf::Rectangle;
use wgpu::util::DeviceExt;
use wgpu::vertex_attr_array;

#[derive(Debug)]
pub struct FilterSource<'a> {
    pub texture: &'a wgpu::Texture,
    pub point: (u32, u32),
    pub size: (u32, u32),
}

impl<'a> FilterSource<'a> {
    pub fn for_entire_texture(texture: &'a wgpu::Texture) -> Self {
        Self {
            texture,
            point: (0, 0),
            size: (texture.width(), texture.height()),
        }
    }

    pub fn vertices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let source_width = self.texture.width() as f32;
        let source_height = self.texture.height() as f32;
        let left = self.point.0;
        let top = self.point.1;
        let right = left + self.size.0;
        let bottom = top + self.size.1;
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: create_debug_label!("Filter vertices").as_deref(),
            contents: bytemuck::cast_slice(&[
                FilterVertex {
                    position: [0.0, 0.0],
                    uv: [left as f32 / source_width, top as f32 / source_height],
                },
                FilterVertex {
                    position: [1.0, 0.0],
                    uv: [right as f32 / source_width, top as f32 / source_height],
                },
                FilterVertex {
                    position: [1.0, 1.0],
                    uv: [right as f32 / source_width, bottom as f32 / source_height],
                },
                FilterVertex {
                    position: [0.0, 1.0],
                    uv: [left as f32 / source_width, bottom as f32 / source_height],
                },
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn vertices_with_blur_offset(
        &self,
        device: &wgpu::Device,
        blur_offset: (f32, f32),
    ) -> wgpu::Buffer {
        let source_width = self.texture.width() as f32;
        let source_height = self.texture.height() as f32;
        let source_left = self.point.0;
        let source_top = self.point.1;
        let source_right = source_left + self.size.0;
        let source_bottom = source_top + self.size.1;
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: create_debug_label!("Filter vertices").as_deref(),
            contents: bytemuck::cast_slice(&[
                FilterVertexWithBlur {
                    position: [0.0, 0.0],
                    source_uv: [
                        source_left as f32 / source_width,
                        source_top as f32 / source_height,
                    ],
                    blur_uv: [
                        (source_left as f32 + blur_offset.0) / source_width,
                        (source_top as f32 + blur_offset.1) / source_height,
                    ],
                },
                FilterVertexWithBlur {
                    position: [1.0, 0.0],
                    source_uv: [
                        source_right as f32 / source_width,
                        source_top as f32 / source_height,
                    ],
                    blur_uv: [
                        (source_right as f32 + blur_offset.0) / source_width,
                        (source_top as f32 + blur_offset.1) / source_height,
                    ],
                },
                FilterVertexWithBlur {
                    position: [1.0, 1.0],
                    source_uv: [
                        source_right as f32 / source_width,
                        source_bottom as f32 / source_height,
                    ],
                    blur_uv: [
                        (source_right as f32 + blur_offset.0) / source_width,
                        (source_bottom as f32 + blur_offset.1) / source_height,
                    ],
                },
                FilterVertexWithBlur {
                    position: [0.0, 1.0],
                    source_uv: [
                        source_left as f32 / source_width,
                        source_bottom as f32 / source_height,
                    ],
                    blur_uv: [
                        (source_left as f32 + blur_offset.0) / source_width,
                        (source_bottom as f32 + blur_offset.1) / source_height,
                    ],
                },
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}

pub struct Filters {
    pub blur: BlurFilter,
    pub color_matrix: ColorMatrixFilter,
    pub shader: ShaderFilter,
    pub glow: GlowFilter,
}

impl Filters {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            blur: BlurFilter::new(device),
            color_matrix: ColorMatrixFilter::new(device),
            shader: ShaderFilter::new(),
            glow: GlowFilter::new(device),
        }
    }

    pub fn calculate_dest_rect(
        &self,
        filter: &Filter,
        source_rect: Rectangle<i32>,
    ) -> Rectangle<i32> {
        match filter {
            Filter::BlurFilter(filter) => self.blur.calculate_dest_rect(filter, source_rect),
            Filter::GlowFilter(filter) => {
                self.glow
                    .calculate_dest_rect(filter, source_rect, &self.blur)
            }
            Filter::DropShadowFilter(filter) => {
                DropShadowFilter::calculate_dest_rect(filter, source_rect, &self.blur, &self.glow)
            }
            _ => source_rect,
        }
    }

    pub fn apply(
        &self,
        descriptors: &Descriptors,
        draw_encoder: &mut wgpu::CommandEncoder,
        texture_pool: &mut TexturePool,
        source: FilterSource,
        filter: Filter,
    ) -> CommandTarget {
        let target = match filter {
            Filter::ColorMatrixFilter(filter) => Some(descriptors.filters.color_matrix.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                &source,
                &filter,
            )),
            Filter::BlurFilter(filter) => descriptors.filters.blur.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                &source,
                &filter,
            ),
            Filter::ShaderFilter(shader) => Some(descriptors.filters.shader.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                &source,
                shader,
            )),
            Filter::GlowFilter(filter) => Some(descriptors.filters.glow.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                &source,
                &filter,
                &self.blur,
                (0.0, 0.0),
            )),
            Filter::DropShadowFilter(filter) => Some(DropShadowFilter::apply(
                descriptors,
                texture_pool,
                draw_encoder,
                &source,
                &filter,
                &self.blur,
                &self.glow,
            )),
            filter => {
                static WARNED_FILTERS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
                let name = match filter {
                    Filter::BevelFilter(_) => "BevelFilter",
                    Filter::GradientGlowFilter(_) => "GradientGlowFilter",
                    Filter::GradientBevelFilter(_) => "GradientBevelFilter",
                    Filter::ConvolutionFilter(_) => "ConvolutionFilter",
                    Filter::DisplacementMapFilter(_) => "DisplacementMapFilter",
                    Filter::ColorMatrixFilter(_)
                    | Filter::BlurFilter(_)
                    | Filter::GlowFilter(_)
                    | Filter::DropShadowFilter(_)
                    | Filter::ShaderFilter(_) => unreachable!(),
                };
                // Only warn once per filter type
                if WARNED_FILTERS
                    .get_or_init(Default::default)
                    .lock()
                    .unwrap()
                    .insert(name)
                {
                    tracing::warn!("Unsupported filter {filter:?}");
                }
                None
            }
        };

        let target = target.unwrap_or_else(|| {
            // Apply a default color matrix - it's essentially a blit
            // TODO: Not need to do this.
            descriptors.filters.color_matrix.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                &source,
                &Default::default(),
            )
        });

        // We're about to perform a copy, so make sure that we've applied
        // a clear (in case no other draw commands were issued, we still need
        // the background clear color applied)
        target.ensure_cleared(draw_encoder);
        target
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FilterVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

pub const VERTEX_BUFFERS_DESCRIPTION_FILTERS: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<FilterVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ],
    }];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FilterVertexWithBlur {
    pub position: [f32; 2],
    pub source_uv: [f32; 2],
    pub blur_uv: [f32; 2],
}

pub const VERTEX_BUFFERS_DESCRIPTION_FILTERS_WITH_BLUR: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<FilterVertexWithBlur>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x2,
        ],
    }];
