use crate::renderer::gpu_primitives::{Index, InstanceRaw, Vertex};
use std::ops::Range;
use wgpu::util::DeviceExt;

pub const MAX_INSTANCES: u64 = 1024;

pub struct Hitbox {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Hitbox {
    pub fn new(device: &mut wgpu::Device) -> Self {
        let (vertex_data, index_data) = create_vertices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsage::INDEX,
        });

        let instance_buf_size = MAX_INSTANCES * std::mem::size_of::<InstanceRaw>() as u64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            size: instance_buf_size,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_indices: index_data.len() as u32,
        }
    }

    pub fn update_instance_buffer(&mut self, instances: Vec<InstanceRaw>, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(instances.as_slice()),
        );
    }
}
pub trait DrawHitbox<'a, 'b>
where
    'b: 'a,
{
    fn draw_hitbox(
        &mut self,
        model: &'b Hitbox,
        instances: Range<u32>,
        uniform_bind_group: &'b wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawHitbox<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_hitbox(
        &mut self,
        model: &'b Hitbox,
        instances: Range<u32>,
        uniform_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, model.vertex_buffer.slice(..));
        self.set_vertex_buffer(1, model.instance_buffer.slice(..));
        self.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.set_bind_group(0, uniform_bind_group, &[]);
        self.draw_indexed(0..model.num_indices, 0, instances);
    }
}
fn create_vertices() -> (Vec<Vertex>, Vec<Index>) {
    let w = 0.5;
    let h = 0.5;
    let vertex_data = [
        Vertex {
            pos: [-w, -h, 0.0, 1.0],
            tex_coord: [0.0, 1.0],
        },
        Vertex {
            pos: [w, -h, 0.0, 1.0],
            tex_coord: [1.0, 1.0],
        },
        Vertex {
            pos: [w, h, 0.0, 1.0],
            tex_coord: [1.0, 0.0],
        },
        Vertex {
            pos: [-w, h, 0.0, 1.0],
            tex_coord: [0.0, 0.0],
        },
    ];

    let index_data: &[u16] = &[0, 1, 2, 3, 0];

    (vertex_data.to_vec(), index_data.to_vec())
}
