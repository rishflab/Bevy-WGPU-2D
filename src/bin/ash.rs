use ash::{util::*, version::DeviceV1_0, vk};
use erlking::{
    self, find_memorytype_index, find_memorytype_index_f, offset_of, record_submit_commandbuffer,
};
use std::{default::Default, ffi::CString, io::Cursor, mem, mem::align_of};
use winit::{event::VirtualKeyCode, event_loop::ControlFlow};

fn main() {
    unsafe {
        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("Erlking")
            .with_min_inner_size(winit::dpi::LogicalSize::new(
                f64::from(1280),
                f64::from(720),
            ))
            .build(&event_loop)
            .unwrap();

        let mut renderer = erlking::Vulkan::new(&window);

        event_loop.run(move |event, _, control_flow| match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit
                    }
                }
                winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            winit::event::Event::MainEventsCleared => renderer.draw(),
            _ => (),
        });
    }
}
