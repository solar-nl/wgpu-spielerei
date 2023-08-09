use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop}, 
};

mod graphics;
use graphics::Graphics;

pub async fn run(title: &str, width: f64, height: f64) {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
    .with_title(title)
    .with_inner_size(winit::dpi::LogicalSize::new(width, height))
    .build(&event_loop)
    .unwrap();

    let mut graphics = Graphics::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == graphics.window().id() => {
                if !graphics.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            graphics.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            graphics.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == graphics.window().id() => {
                graphics.update();
                match graphics.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        graphics.resize(graphics.window().inner_size())
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                graphics.window().request_redraw();
            }
            _ => {}
        }
    });
}