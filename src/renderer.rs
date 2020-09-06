use crate::{find_memorytype_index, Vertex};
use ash::{
    util::{read_spv, Align},
    version::DeviceV1_0,
    vk, Device,
};
use std::{ffi::CString, io::Cursor, mem, mem::align_of};

// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::zeroed();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}

pub struct Renderer {
    pub renderpass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub graphics_pipelines: Vec<vk::Pipeline>,
    pub viewport: [vk::Viewport; 1],
    pub index_buffer_memory: vk::DeviceMemory,
    pub vertex_input_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub vertex_input_buffer: vk::Buffer,
    pub scissors: [vk::Rect2D; 1],
    pub index_buffer_data: [u32; 3],
    pub pipeline_layout: vk::PipelineLayout,
    pub vertex_shader_module: vk::ShaderModule,
    pub fragment_shader_module: vk::ShaderModule,
}

impl Renderer {
    pub fn new(
        device: &mut Device,
        surface_format: vk::SurfaceFormatKHR,
        present_image_views: &mut Vec<vk::ImageView>,
        depth_image_view: vk::ImageView,
        surface_resolution: vk::Extent2D,
        device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        unsafe {
            let renderpass_attachments = [
                vk::AttachmentDescription {
                    format: surface_format.format,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    store_op: vk::AttachmentStoreOp::STORE,
                    final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                    ..Default::default()
                },
                vk::AttachmentDescription {
                    format: vk::Format::D16_UNORM,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    ..Default::default()
                },
            ];
            let color_attachment_refs = [vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            }];
            let depth_attachment_ref = vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            };
            let dependencies = [vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            }];

            let subpasses = [vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()];

            let renderpass_create_info = vk::RenderPassCreateInfo::builder()
                .attachments(&renderpass_attachments)
                .subpasses(&subpasses)
                .dependencies(&dependencies);

            let renderpass = device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap();

            let framebuffers: Vec<vk::Framebuffer> = present_image_views
                .iter()
                .map(|&present_image_view| {
                    let framebuffer_attachments = [present_image_view, depth_image_view];
                    let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(renderpass)
                        .attachments(&framebuffer_attachments)
                        .width(surface_resolution.width)
                        .height(surface_resolution.height)
                        .layers(1);

                    device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap()
                })
                .collect();

            let index_buffer_data = [0u32, 1, 2];
            let index_buffer_info = vk::BufferCreateInfo::builder()
                .size(std::mem::size_of_val(&index_buffer_data) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let index_buffer = device.create_buffer(&index_buffer_info, None).unwrap();
            let index_buffer_memory_req = device.get_buffer_memory_requirements(index_buffer);
            let index_buffer_memory_index = find_memorytype_index(
                &index_buffer_memory_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("Unable to find suitable memorytype for the index buffer.");

            let index_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: index_buffer_memory_req.size,
                memory_type_index: index_buffer_memory_index,
                ..Default::default()
            };
            let index_buffer_memory = device.allocate_memory(&index_allocate_info, None).unwrap();
            let index_ptr = device
                .map_memory(
                    index_buffer_memory,
                    0,
                    index_buffer_memory_req.size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            let mut index_slice = Align::new(
                index_ptr,
                align_of::<u32>() as u64,
                index_buffer_memory_req.size,
            );
            index_slice.copy_from_slice(&index_buffer_data);
            device.unmap_memory(index_buffer_memory);

            device
                .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
                .unwrap();

            let vertex_input_buffer_info = vk::BufferCreateInfo {
                size: 3 * std::mem::size_of::<Vertex>() as u64,
                usage: vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };

            let vertex_input_buffer = device
                .create_buffer(&vertex_input_buffer_info, None)
                .unwrap();

            let vertex_input_buffer_memory_req =
                device.get_buffer_memory_requirements(vertex_input_buffer);

            let vertex_input_buffer_memory_index = find_memorytype_index(
                &vertex_input_buffer_memory_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("Unable to find suitable memorytype for the vertex buffer.");

            let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: vertex_input_buffer_memory_req.size,
                memory_type_index: vertex_input_buffer_memory_index,
                ..Default::default()
            };

            let vertex_input_buffer_memory = device
                .allocate_memory(&vertex_buffer_allocate_info, None)
                .unwrap();

            let vertices = [
                Vertex {
                    pos: [-1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                Vertex {
                    pos: [1.0, 1.0, 0.0, 1.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                Vertex {
                    pos: [0.0, -1.0, 0.0, 1.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
            ];

            let vert_ptr = device
                .map_memory(
                    vertex_input_buffer_memory,
                    0,
                    vertex_input_buffer_memory_req.size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();

            let mut vert_align = Align::new(
                vert_ptr,
                align_of::<Vertex>() as u64,
                vertex_input_buffer_memory_req.size,
            );
            vert_align.copy_from_slice(&vertices);
            device.unmap_memory(vertex_input_buffer_memory);

            device
                .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)
                .unwrap();
            let mut vertex_spv_file = Cursor::new(&include_bytes!("../shaders/vert.spv")[..]);
            let mut frag_spv_file = Cursor::new(&include_bytes!("../shaders/frag.spv")[..]);

            let vertex_code =
                read_spv(&mut vertex_spv_file).expect("Failed to read vertex shader spv file");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);

            let frag_code =
                read_spv(&mut frag_spv_file).expect("Failed to read fragment shader spv file");
            let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

            let vertex_shader_module = device
                .create_shader_module(&vertex_shader_info, None)
                .expect("Vertex shader module error");

            let fragment_shader_module = device
                .create_shader_module(&frag_shader_info, None)
                .expect("Fragment shader module error");

            let layout_create_info = vk::PipelineLayoutCreateInfo::default();

            let pipeline_layout = device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap();

            let shader_entry_name = CString::new("main").unwrap();
            let shader_stage_create_infos = [
                vk::PipelineShaderStageCreateInfo {
                    module: vertex_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::PipelineShaderStageCreateInfo {
                    s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                    module: fragment_shader_module,
                    p_name: shader_entry_name.as_ptr(),
                    stage: vk::ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                },
            ];
            let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
                binding: 0,
                stride: mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }];
            let vertex_input_attribute_descriptions = [
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(Vertex, pos) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(Vertex, color) as u32,
                },
            ];

            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
                vertex_attribute_description_count: vertex_input_attribute_descriptions.len()
                    as u32,
                p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
                vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
                p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
                ..Default::default()
            };
            let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            };
            let viewport = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: surface_resolution.width as f32,
                height: surface_resolution.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];
            let scissors = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_resolution,
            }];
            let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
                .scissors(&scissors)
                .viewports(&viewport);

            let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
                front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                line_width: 1.0,
                polygon_mode: vk::PolygonMode::FILL,
                ..Default::default()
            };
            let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
                rasterization_samples: vk::SampleCountFlags::TYPE_1,
                ..Default::default()
            };
            let noop_stencil_state = vk::StencilOpState {
                fail_op: vk::StencilOp::KEEP,
                pass_op: vk::StencilOp::KEEP,
                depth_fail_op: vk::StencilOp::KEEP,
                compare_op: vk::CompareOp::ALWAYS,
                ..Default::default()
            };
            let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
                depth_test_enable: 1,
                depth_write_enable: 1,
                depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
                front: noop_stencil_state,
                back: noop_stencil_state,
                max_depth_bounds: 1.0,
                ..Default::default()
            };
            let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::all(),
            }];
            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_states);

            let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state_info =
                vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

            let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stage_create_infos)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .depth_stencil_state(&depth_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .render_pass(renderpass);

            let graphics_pipelines = device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info.build()],
                    None,
                )
                .expect("Unable to create graphics pipeline");

            Self {
                renderpass,
                framebuffers,
                graphics_pipelines,
                viewport,
                index_buffer,
                vertex_input_buffer,
                scissors,
                index_buffer_data,
                pipeline_layout,
                index_buffer_memory,
                vertex_shader_module,
                fragment_shader_module,
                vertex_input_buffer_memory,
            }
        }
    }
}
