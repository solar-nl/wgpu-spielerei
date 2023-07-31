use wgpu::{*, util::{BufferInitDescriptor, DeviceExt,}};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

use std::{
    time::Instant,
    time::Duration,
    io::BufReader,
};

mod resource;
mod commandbuffer;
use commandbuffer:: {Command,CommandBuffer};

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
/* 
          Some(VirtualKeyCode::Up) => Some(Command::IncreaseVolume),
          Some(VirtualKeyCode::Down) => Some(Command::DecreaseVolume),
*/
          _ => None,
      }
  } else {
      None
  }
}

// Use a fixed time step for  logic updates.
const FIXED_TIME_STEP: Duration = Duration::from_millis(16);


fn initialize_window(event_loop: &EventLoop<()>) -> Window {
  WindowBuilder::new()
    .with_title("Solar Assembly 2024 Winner Demo")
    .with_inner_size(winit::dpi::LogicalSize::new(960.0, 540.0))
    .build(event_loop).unwrap()
}


 fn main() {
    env_logger::init(); // Necessary for logging within WGPU
    let event_loop = EventLoop::new(); // Loop provided by winit for handling window events
    let window = initialize_window(&event_loop);
 
    let mut frame_count = 0;
    let mut command_buffer = CommandBuffer::new();
    let mut last_update_time = Instant::now();

    let mut _playback_volume: f32 = 1.0;

    let instance = Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });    
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
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

     let surface_caps = surface.get_capabilities(&adapter);
     // Shader code in this tutorial assumes an Srgb surface texture. Using a different
     // one will result all the colors comming out darker. If you want to support non
     // Srgb surfaces, you'll need to account for that when drawing to the frame.
     let surface_format = surface_caps.formats.iter()
         .copied().find(|f| f.is_srgb())
         .unwrap_or(surface_caps.formats[0]);
     let config = wgpu::SurfaceConfiguration {
         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
         format: surface_format,
         width: size.width,
         height: size.height,
         present_mode: surface_caps.present_modes[0],
         alpha_mode: surface_caps.alpha_modes[0],
         view_formats: vec![],
     };
     

      
      surface.configure(&device, &config);

      let solar_logo_bytes = include_bytes!("solar_groot.jpg");
      let solar_logo_texture =  resource::Texture::new(&device, &queue, solar_logo_bytes, Some("Solar Logo")).unwrap();

      let texture_descriptor = wgpu::TextureDescriptor {
        size: wgpu::Extent3d{width: window.inner_size().width, height: window.inner_size().height, depth_or_array_layers: 1},
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,//wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        label: None,
        view_formats: &[], // Default
      };

      let texture_view_descriptor: TextureViewDescriptor = wgpu::TextureViewDescriptor {
        label: None,
        format: config.format.into(),//Some(wgpu::TextureFormat::Bgra8UnormSrgb),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
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
    
    let rt_0_view = rt_0.create_view(&Default::default());
    let rt_1_view = rt_1.create_view(&Default::default());
    let rt_2_view = rt_2.create_view(&Default::default());
    let _rt_3_view = rt_3.create_view(&texture_view_descriptor);

    let render_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: None,
      address_mode_u: wgpu::AddressMode::Repeat,
      address_mode_v: wgpu::AddressMode::Repeat,
      address_mode_w: wgpu::AddressMode::Repeat,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });
     
    let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("not_menger.wgsl").into()),
    });

    let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("not_menger.wgsl").into()),
    });

    let mut uniforms = Uniforms { resolution: [size.width as _, size.height as _], time: 0., i_pass: 0 };
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
              resource: wgpu::BindingResource::TextureView(&rt_1_view),
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
            resource: wgpu::BindingResource::TextureView(&solar_logo_texture.view),
        }, 
      ],
      layout: &bind_group_layout,
      label: Some("bind group"),
  });

  let bind_group_rt = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
          resource: wgpu::BindingResource::TextureView(&solar_logo_texture.view),
      }, 
    ],
    layout: &bind_group_layout,
    label: Some("bind group rt"),
});


  let render_pipeline_layout =
  device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
          targets: &[Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })],        
      }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },  
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
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encode 1r"),
                });
     
                {
                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                          view: &rt_0_view, 
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
                      
                      
                      /*for i in 0..1*/ {
                        uniforms.i_pass = 0; // i
                        queue.write_buffer(&uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));
                        render_pass.draw(0..3, 0..1);
                      }
                }
 
                queue.submit(std::iter::once(encoder.finish()));
                let mut encoder2 = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                  label: Some("Render Encoder 2"),
                });

                {
                  let mut render_pass = encoder2.begin_render_pass(&RenderPassDescriptor {
                      label: None,
                      color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view, 
                        resolve_target: None,
                        ops: Operations {
                          load: LoadOp::Clear(Color::BLACK),
                          store: true,
                        },
                      })],
                      depth_stencil_attachment: None,
                    });                
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &bind_group_rt, &[]);
                    
                    
                    /*for i in 1..2*/ {
                      uniforms.i_pass = 1; // i
                      queue.write_buffer(&uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));
                      render_pass.draw(0..3, 0..1);
                    }
              }

                queue.submit(std::iter::once(encoder2.finish()));
                output.present();
              },
            // New
            Event::MainEventsCleared => {
              while let Some(command) = command_buffer.next_command() {
                  match command {
                      Command::Quit => {
                          *control_flow = quit()
                      }
                      Command::Play => {
                          println!("Play");
                          println!("framecount: {}",frame_count);
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
                      }
                      //_ => ()
                  }
                  
              }

              let now = Instant::now();
              let time_delta = now.duration_since(last_update_time);

              if time_delta >= FIXED_TIME_STEP {
                  // Update logic here.
                  last_update_time = now;

                  frame_count+=1;

              }

              window.request_redraw();
            }

            Event::WindowEvent {
              event: WindowEvent::CloseRequested,
              window_id,
          } if window_id == window.id() => command_buffer.add_command(Command::Quit),
          Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {input, .. },
                ..
        } => {
            if let Some(command) = handle_keyboard_input(input) {
                command_buffer.add_command(command);
            }
        }
        // handle mouse input here
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            println!("Mouse moved to ({:?})", position);
        }
        // handle mouse button input here
        Event::WindowEvent {
          event: WindowEvent::MouseInput { state, button, .. },
          ..
      } => {
          println!("Mouse button {:?} is {:?} at {:?}", button, state, Instant::now());
      }
      // handle mouse wheel input here
      Event::WindowEvent {
          event: WindowEvent::MouseWheel { delta, .. },
          ..
      } => {
          println!("Mouse wheel moved {:?} at {:?}", delta, Instant::now());
      }

        _ => (),
        }

        
    });
}

fn quit () -> ControlFlow {
  ControlFlow::Exit
}