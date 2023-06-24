use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod commandbuffer;
use commandbuffer::{Command, CommandBuffer};

use std::{io::BufReader, time::Duration, time::Instant};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    i_pass: i32,
}

fn handle_keyboard_input(input: KeyboardInput) -> Option<Command> {
    if input.state == ElementState::Released {
        match input.virtual_keycode {
            Some(VirtualKeyCode::Escape) => Some(Command::Quit),
            Some(VirtualKeyCode::J) => Some(Command::PlayReverse),
            Some(VirtualKeyCode::K) => Some(Command::Pause),
            Some(VirtualKeyCode::L) => Some(Command::PlayForward),
            Some(VirtualKeyCode::Space) => Some(Command::Play),
            Some(VirtualKeyCode::Grave) => Some(Command::DebugDraw),
            _ => None,
        }
    } else {
        None
    }
}

// Use a fixed time step for  logic updates.
const FIXED_TIME_STEP: Duration = Duration::from_millis(16);

fn main() {
    env_logger::init(); // Necessary for logging within WGPU
    let event_loop = EventLoop::new(); // Loop provided by winit for handling window events
    let window = WindowBuilder::new()
        .with_title("Solar Assembly 2024 Winner Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(960.0, 540.0))
        .build(&event_loop)
        .unwrap();

    let mut frame_count = 0;
    let mut command_buffer = CommandBuffer::new();

    let mut last_update_time = Instant::now();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        None, // Trace path
    ))
    .unwrap();

    let size = window.inner_size();
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        alpha_mode: CompositeAlphaMode::Auto,
        present_mode: PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    let texture_descriptor = wgpu::TextureDescriptor {
        size: wgpu::Extent3d::default(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: None,
    };

    let rt_0 = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        ..texture_descriptor
    });
    let rt_1 = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        ..texture_descriptor
    });
    let rt_2 = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        ..texture_descriptor
    });
    let rt_3 = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        ..texture_descriptor
    });

    let rt_0_view = rt_0.create_view(&wgpu::TextureViewDescriptor::default());
    let rt_1_view = rt_1.create_view(&wgpu::TextureViewDescriptor::default());
    let rt_2_view = rt_2.create_view(&wgpu::TextureViewDescriptor::default());
    let rt_3_view = rt_3.create_view(&wgpu::TextureViewDescriptor::default());

    let render_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let mut uniforms = Uniforms {
        resolution: [size.width.clone() as _, size.height.clone() as _],
        time: 0.,
        i_pass: 0,
    };
    let time = Instant::now();
    let uniforms_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::bytes_of(&uniforms),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&render_texture_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&rt_0_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&rt_1_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&rt_2_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&rt_3_view),
            },
        ],
        layout: &bind_group_layout,
        label: Some("bind group"),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &vertex_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &fragment_shader,
            entry_point: "fs_main",
            targets: &[Some(config.format.into())],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    });

    // Audio goes here
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let file = std::fs::File::open("music.mp3").unwrap();
    let _music = stream_handle.play_once(BufReader::new(file)).unwrap();

    // Opens the window and starts processing events
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let _time_delta = now.duration_since(last_update_time);

                uniforms.time = time.elapsed().as_secs_f32();
                queue.write_buffer(&uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));

                let output = surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::BLUE),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);

                    for i in 0..1 {
                        uniforms.i_pass = i;
                        queue.write_buffer(&uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));
                        render_pass.draw(0..3, 0..1);
                    }
                }

                // submit will accept anything that implements IntoIter
                queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            // New
            Event::MainEventsCleared => {
                while let Some(command) = command_buffer.next_command() {
                    match command {
                        Command::Quit => {
                            println!("framecount: {}", frame_count);
                            *control_flow = ControlFlow::Exit
                        }
                        Command::Play => {
                            println!("Play");
                            println!("framecount: {}", frame_count);
                        }
                        Command::DebugDraw => {
                            println!("DebugDraw")
                        }
                        Command::Pause => {
                            println!("Pause!")
                        }
                        Command::PlayForward => {
                            println!("PlayForward!")
                        }
                        Command::PlayReverse => {
                            println!("PlayReverse!")
                        } //_ => ()
                    }
                }

                let now = Instant::now();
                let time_delta = now.duration_since(last_update_time);

                if time_delta >= FIXED_TIME_STEP {
                    // Update logic here.
                    last_update_time = now;

                    frame_count += 1;
                }

                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => command_buffer.add_command(Command::Quit),
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(command) = handle_keyboard_input(input) {
                    command_buffer.add_command(command);
                }
            }
            _ => (),
        }
    });
}
