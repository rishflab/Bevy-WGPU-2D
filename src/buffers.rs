use ash::{util::Align, version::DeviceV1_0, vk, Device};
use failure::_core::mem::align_of;

pub struct Buffer(vk::Buffer);

impl Buffer {
    pub unsafe fn new(
        device: &mut Device,
        buffer_size: u64,
        memory: vk::DeviceMemory,
        offset: u64,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
    ) {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(usage)
            .sharing_mode(sharing_mode);

        let buffer = device.create_buffer(&buffer_info, None).unwrap();

        Self(device.bind_buffer_memory(buffer, memory, offset).unwrap())
    }
}
