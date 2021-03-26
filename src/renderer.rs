use std::{mem, num::NonZeroU32};

use wgpu::{util::DeviceExt, BlendFactor, BlendOperation};

use gpu_primitives::{CameraUniform, InstanceRaw, Vertex};
use scene::Scene;
use sprite::{DrawSprite, Sprite};
use texture::DepthTexture;

use crate::asset::SpriteData;
use crate::renderer::hitbox::{DrawHitbox, Hitbox};

pub mod gpu_primitives;
mod hitbox;
pub mod scene;
pub mod sprite;
pub mod texture;

pub const TEXTURE_ARRAY_SIZE: usize = 128;

pub struct Renderer {
    sprites: Vec<Sprite>,
    hitboxes: Vec<Hitbox>,
    uniform_buffer: wgpu::Buffer,
    sprite_pipeline: wgpu::RenderPipeline,
    hitbox_pipeline: wgpu::RenderPipeline,
    depth_texture: DepthTexture,
    uniform_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        queue: &wgpu::Queue,
        sprite_assets: Vec<SpriteData>,
    ) -> Self {
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: &[0u8; mem::size_of::<CameraUniform>()],
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<CameraUniform>() as _
                        ),
                        has_dynamic_offset: false,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let sprite_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: Some(
                            NonZeroU32::new(TEXTURE_ARRAY_SIZE as u32)
                                .expect("array texture count should be a positive number"),
                        ),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: false,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });

        let mut sprites = vec![];

        for asset in sprite_assets {
            sprites.push(Sprite::new(
                device,
                queue,
                &sprite_bind_group_layout,
                asset.id,
                asset.frames,
            ));
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout, &sprite_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/shader.vert.spv"));
        let fs_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/shader.frag.spv"));
        let wire_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/wire.frag.spv"));

        let depth_texture = DepthTexture::new(&device, &sc_desc);

        let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendState {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendState {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Min,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState::default(),
        });

        let hitbox_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &wire_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendState {
                        operation: wgpu::BlendOperation::Add,
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    },
                    alpha_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Line,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState::default(),
        });

        Renderer {
            uniform_buffer,
            sprite_pipeline,
            hitbox_pipeline,
            sprites,
            depth_texture,
            uniform_bind_group,
            hitboxes: vec![Hitbox::new(device, "cuboid".to_string())],
        }
    }

    pub fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _sc_desc: &wgpu::SwapChainDescriptor,
        scene: Scene,
    ) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&scene.camera_uniform),
        );

        for hitbox in self.hitboxes.iter_mut() {
            if let Some(instances) = scene.hitbox_instances.get(&hitbox.id) {
                hitbox.update_instance_buffer(instances.clone(), queue);
            }
        }

        for sprite in self.sprites.iter_mut() {
            if let Some(instances) = scene.sprite_instances.get(&sprite.id) {
                sprite.update_instance_buffer(instances.clone(), queue);
            }
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_pipeline(&self.sprite_pipeline);

            for sprite in self.sprites.iter() {
                if let Some(instances) = scene.sprite_instances.get(&sprite.id) {
                    rpass.draw_sprite(sprite, 0..instances.len() as u32, &self.uniform_bind_group);
                }
            }

            rpass.set_pipeline(&self.hitbox_pipeline);

            // // render sprite outlines
            // for sprite in self.sprites.iter() {
            //     if let Some(instances) = scene.sprite_instances.get(&sprite.id) {
            //         rpass.draw_sprite(sprite, 0..instances.len() as u32,
            // &self.uniform_bind_group);     }
            // }

            for hitbox in self.hitboxes.iter() {
                if let Some(instances) = scene.hitbox_instances.get(&hitbox.id) {
                    rpass.draw_hitbox(hitbox, 0..instances.len() as u32, &self.uniform_bind_group);
                }
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
