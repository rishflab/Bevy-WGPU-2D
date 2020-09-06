// extern crate erlking;
// use wgpu::util::DeviceExt;
//
// use erlking::{
//     asset::{MeshData, StaticMesh, StaticMeshHandle, Vertex},
//     camera::Camera,
//     framework,
//     game::World,
// };
// use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
//
// fn create_texels(size: usize) -> Vec<u8> {
//     use std::iter;
//
//     (0..size * size)
//         .flat_map(|id| {
//             // get high five for recognizing this ;)
//             let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
//             let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
//             let (mut x, mut y, mut count) = (cx, cy, 0);
//             while count < 0xFF && x * x + y * y < 4.0 {
//                 let old_x = x;
//                 x = x * x - y * y + cx;
//                 y = 2.0 * old_x * y + cy;
//                 count += 1;
//             }
//             iter::once(0xFF - (count * 5) as u8)
//                 .chain(iter::once(0xFF - (count * 15) as u8))
//                 .chain(iter::once(0xFF - (count * 50) as u8))
//                 .chain(iter::once(1))
//         })
//         .collect()
// }
//
// struct Example {
//     world: World,
//     vertex_buf: wgpu::Buffer,
//     index_buf: wgpu::Buffer,
//     instance_buf: wgpu::Buffer,
//     bind_group: wgpu::BindGroup,
//     uniform_buf: wgpu::Buffer,
//     pipeline: wgpu::RenderPipeline,
//     camera: Camera,
//     static_mesh_handles: Vec<StaticMeshHandle>,
// }
//
// impl framework::Framework for Example {
//     fn init(
//         sc_desc: &wgpu::SwapChainDescriptor,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//     ) -> Self {
//         use std::mem;
//
//         let world = World::test();
//
//         let icosphere_path = std::path::Path::new(&"assets/icosphere.gltf");
//         let icosphere_mesh = StaticMesh::new(icosphere_path);
//
//         let cube_path = std::path::Path::new(&"assets/cube.gltf");
//         let cube_mesh = StaticMesh::new(cube_path);
//
//         let mut mesh_data = MeshData::new();
//         mesh_data.insert_static_mesh(cube_mesh);
//         mesh_data.insert_static_mesh(icosphere_mesh);
//
//         let vertex_buf =
// device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// label: Some("Vertex Buffer"),             contents:
// bytemuck::cast_slice(&mesh_data.vertex_data()),             usage:
// wgpu::BufferUsage::VERTEX,         });
//
//         let index_buf =
// device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// label: Some("Index Buffer"),             contents:
// bytemuck::cast_slice(&mesh_data.index_data()),             usage:
// wgpu::BufferUsage::INDEX,         });
//
//         let (instance_data, _) = world.instance_data();
//
//         let instance_buf =
// device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// label: Some("Instance Buffer"),             contents:
// bytemuck::cast_slice(&instance_data),             usage:
// wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,         });
//
//         // Create pipeline layout
//         let bind_group_layout =
// device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: None,
//             entries: &[
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 0,
//                     visibility: wgpu::ShaderStage::VERTEX,
//                     ty: wgpu::BindingType::UniformBuffer {
//                         dynamic: false,
//                         min_binding_size: wgpu::BufferSize::new(64),
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 1,
//                     visibility: wgpu::ShaderStage::VERTEX,
//                     ty: wgpu::BindingType::StorageBuffer {
//                         dynamic: false,
//                         min_binding_size: None,
//                         readonly: false,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 2,
//                     visibility: wgpu::ShaderStage::FRAGMENT,
//                     ty: wgpu::BindingType::SampledTexture {
//                         multisampled: false,
//                         component_type: wgpu::TextureComponentType::Float,
//                         dimension: wgpu::TextureViewDimension::D2,
//                     },
//                     count: None,
//                 },
//                 wgpu::BindGroupLayoutEntry {
//                     binding: 3,
//                     visibility: wgpu::ShaderStage::FRAGMENT,
//                     ty: wgpu::BindingType::Sampler { comparison: false },
//                     count: None,
//                 },
//             ],
//         });
//         let pipeline_layout =
// device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
// label: None,             bind_group_layouts: &[&bind_group_layout],
//             push_constant_ranges: &[],
//         });
//
//         // Create the texture
//         let size = 256u32;
//         let texels = create_texels(size as usize);
//         let texture_extent = wgpu::Extent3d {
//             width: size,
//             height: size,
//             depth: 1,
//         };
//         let texture = device.create_texture(&wgpu::TextureDescriptor {
//             label: None,
//             size: texture_extent,
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rgba8UnormSrgb,
//             usage: wgpu::TextureUsage::SAMPLED |
// wgpu::TextureUsage::COPY_DST,         });
//         let texture_view =
// texture.create_view(&wgpu::TextureViewDescriptor::default());         queue.
// write_texture(             wgpu::TextureCopyView {
//                 texture: &texture,
//                 mip_level: 0,
//                 origin: wgpu::Origin3d::ZERO,
//             },
//             &texels,
//             wgpu::TextureDataLayout {
//                 offset: 0,
//                 bytes_per_row: 4 * size,
//                 rows_per_image: 0,
//             },
//             texture_extent,
//         );
//
//         // Create other resources
//         let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//             address_mode_u: wgpu::AddressMode::ClampToEdge,
//             address_mode_v: wgpu::AddressMode::ClampToEdge,
//             address_mode_w: wgpu::AddressMode::ClampToEdge,
//             mag_filter: wgpu::FilterMode::Nearest,
//             min_filter: wgpu::FilterMode::Linear,
//             mipmap_filter: wgpu::FilterMode::Nearest,
//             ..Default::default()
//         });
//
//         let camera = Camera::new(
//             cgmath::Point3::new(0.0, -20.0, 3.0),
//             cgmath::Point3::new(0f32, 0.0, 0.0),
//         );
//         let mx_total = camera.generate_matrix(sc_desc.width as f32 /
// sc_desc.height as f32);         let mx_ref: &[f32; 16] = mx_total.as_ref();
//         let uniform_buf =
// device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// label: Some("Uniform Buffer"),             contents:
// bytemuck::cast_slice(mx_ref),             usage: wgpu::BufferUsage::UNIFORM |
// wgpu::BufferUsage::COPY_DST,         });
//
//         // Create bind group
//         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
// {             layout: &bind_group_layout,
//             entries: &[
//                 wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: uniform_buf.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 1,
//                     resource: instance_buf.as_entire_binding(),
//                 },
//                 wgpu::BindGroupEntry {
//                     binding: 2,
//                     resource:
// wgpu::BindingResource::TextureView(&texture_view),                 },
//                 wgpu::BindGroupEntry {
//                     binding: 3,
//                     resource: wgpu::BindingResource::Sampler(&sampler),
//                 },
//             ],
//             label: None,
//         });
//
//         // Create the render pipeline
//         let vs_module =
//
// device.create_shader_module(wgpu::include_spirv!("../../assets/shader.vert.
// spv"));         let fs_module =
//
// device.create_shader_module(wgpu::include_spirv!("../../assets/shader.frag.
// spv"));
//
//         let pipeline =
// device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
// label: None,             layout: Some(&pipeline_layout),
//             vertex_stage: wgpu::ProgrammableStageDescriptor {
//                 module: &vs_module,
//                 entry_point: "main",
//             },
//             fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
//                 module: &fs_module,
//                 entry_point: "main",
//             }),
//             rasterization_state: Some(wgpu::RasterizationStateDescriptor {
//                 front_face: wgpu::FrontFace::Ccw,
//                 cull_mode: wgpu::CullMode::Back,
//                 ..Default::default()
//             }),
//             primitive_topology: wgpu::PrimitiveTopology::TriangleList,
//             color_states: &[wgpu::ColorStateDescriptor {
//                 format: sc_desc.format,
//                 color_blend: wgpu::BlendDescriptor::REPLACE,
//                 alpha_blend: wgpu::BlendDescriptor::REPLACE,
//                 write_mask: wgpu::ColorWrite::ALL,
//             }],
//             depth_stencil_state: None,
//             vertex_state: wgpu::VertexStateDescriptor {
//                 index_format: wgpu::IndexFormat::Uint16,
//                 vertex_buffers: &[wgpu::VertexBufferDescriptor {
//                     stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
//                     step_mode: wgpu::InputStepMode::Vertex,
//                     attributes: &[
//                         wgpu::VertexAttributeDescriptor {
//                             format: wgpu::VertexFormat::Float4,
//                             offset: 0,
//                             shader_location: 0,
//                         },
//                         wgpu::VertexAttributeDescriptor {
//                             format: wgpu::VertexFormat::Float2,
//                             offset: 4 * 4,
//                             shader_location: 1,
//                         },
//                     ],
//                 }],
//             },
//             sample_count: 1,
//             sample_mask: !0,
//             alpha_to_coverage_enabled: false,
//         });
//
//         // Done
//         Example {
//             world,
//             vertex_buf,
//             index_buf,
//             bind_group,
//             uniform_buf,
//             pipeline,
//             instance_buf,
//             camera,
//             static_mesh_handles: mesh_data.static_mesh_handles(),
//         }
//     }
//
//     fn input(&mut self, event: winit::event::WindowEvent) {
//         if let WindowEvent::KeyboardInput { input, .. } = event {
//             match input {
//                 KeyboardInput {
//                     state: ElementState::Pressed,
//                     virtual_keycode: Some(VirtualKeyCode::Left),
//                     ..
//                 } => {
//                     self.camera.eye += cgmath::vec3(0.1, 0.0, 0.0);
//                     self.camera.look_at += cgmath::vec3(0.1, 0.0, 0.0);
//                 }
//                 KeyboardInput {
//                     state: ElementState::Pressed,
//                     virtual_keycode: Some(VirtualKeyCode::Right),
//                     ..
//                 } => {
//                     self.camera.eye += cgmath::vec3(-0.1, 0.0, 0.0);
//                     self.camera.look_at += cgmath::vec3(-0.1, 0.0, 0.0);
//                 }
//                 _ => (),
//             }
//         }
//     }
//
//     fn resize(
//         &mut self,
//         sc_desc: &wgpu::SwapChainDescriptor,
//         _device: &wgpu::Device,
//         queue: &wgpu::Queue,
//     ) {
//         let mx_total = self
//             .camera
//             .generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
//         let mx_ref: &[f32; 16] = mx_total.as_ref();
//         queue.write_buffer(&self.uniform_buf, 0,
// bytemuck::cast_slice(mx_ref));     }
//
//     fn update(&mut self) {
//         self.world.update();
//     }
//
//     fn render(
//         &mut self,
//         frame: &wgpu::SwapChainTexture,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         _spawner: &impl futures::task::LocalSpawn,
//         sc_desc: &wgpu::SwapChainDescriptor,
//     ) {
//         let mx_total = self
//             .camera
//             .generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
//         let mx_ref: &[f32; 16] = mx_total.as_ref();
//         queue.write_buffer(&self.uniform_buf, 0,
// bytemuck::cast_slice(mx_ref));
//
//         let (instance_data, instance_ranges) = self.world.instance_data();
//         queue.write_buffer(&self.instance_buf, 0,
// bytemuck::cast_slice(&instance_data));
//
//         let mut encoder =
//             device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
// label: None });         {
//             let mut rpass =
// encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
// color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
// attachment: &frame.view,                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(wgpu::Color {
//                             r: 0.1,
//                             g: 0.2,
//                             b: 0.3,
//                             a: 1.0,
//                         }),
//                         store: true,
//                     },
//                 }],
//                 depth_stencil_attachment: None,
//             });
//             rpass.push_debug_group("Prepare data for draw.");
//             rpass.set_pipeline(&self.pipeline);
//             rpass.set_bind_group(0, &self.bind_group, &[]);
//             rpass.set_index_buffer(self.index_buf.slice(..));
//             rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
//             rpass.pop_debug_group();
//             rpass.insert_debug_marker("Draw!");
//
//             for (handle, range) in
// self.static_mesh_handles.iter().zip(instance_ranges.iter()) {
// rpass.draw_indexed(handle.indices.clone(), handle.base_vertex,
// range.clone());             }
//         }
//
//         queue.submit(Some(encoder.finish()));
//     }
// }

fn main() {
    // framework::run::<Example>("cube");
}
