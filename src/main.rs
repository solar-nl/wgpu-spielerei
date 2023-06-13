use wgpu::{*, util::{BufferInitDescriptor, DeviceExt}};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::{
    time::Instant,
};


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    padding: f32,
}

const VS: &str = "\
struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords : vec2<f32>,
};
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
  
    let u = f32(i32((in_vertex_index << 1u) & 2u));
    let v = f32(i32(in_vertex_index & 2u));

    out.position = vec4<f32>(u * 2.0 - 1.0, v * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords = vec2<f32>(u, v);
    
    return out;
}";

const FS: &str = "\
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    resolution : vec2<f32>,
    time : f32
};

let epsilon = 0.0001;


@group(0) @binding(0) var<uniform> u: Uniforms;

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn sdSphere(pos: vec3<f32>, radius: f32) -> f32 {
    return length(pos) - radius;
}

fn map(pos: vec3<f32>) -> f32 {

    var p = pos;
    p.z -= 4.0;
    var c = 0.8 * cos(u.time);
    let s = 0.2;
                      
    var ball1 = sdSphere(p + vec3<f32>(c, 0.0, 0.0), s);
    var ball2 = sdSphere(p + vec3<f32>(0.0, c, 0.0), s);
    var ball3 = sdSphere(p + vec3<f32>(0.0, 0.0, c), s);
    var ball4 = sdSphere(p - vec3<f32>(c, 0.0, 0.0), s);
    var ball5 = sdSphere(p - vec3<f32>(0.0, c, 0.0), s);
    var ball6 = sdSphere(p - vec3<f32>(0.0, 0.0, c), s);
    
    let t = 0.8;
    ball1 = smin(ball1, ball2, t);
    ball1 = smin(ball1, ball3, t);
    ball1 = smin(ball1, ball4, t);
    ball1 = smin(ball1, ball5, t);
    ball1 = smin(ball1, ball6, t);  
    
    return ball1;
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
	var dist = map(p);
	return normalize(vec3<f32>(map(p + vec3<f32>(epsilon, 0.0, 0.0)) - dist,
                          map(p + vec3<f32>(0.0, epsilon, 0.0)) - dist,
                          map(p + vec3<f32>(0.0, 0.0, epsilon)) - dist));
}


fn raymarch(origin: vec3<f32>, direction: vec3<f32>) -> f32 {
    var t = 0.0;

    for(var i: i32 = 0; i < 40; i++) {
    	let pos = vec3<f32>(origin + t * direction);
        var dist = map(pos);

        t += dist;

        if (dist < epsilon) {
            return t - epsilon;
        }
      }

    return 0.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (in.tex_coords.xy * u.resolution.xy * 2.0 - u.resolution.xy) / u.resolution.x;
    
    let ro = vec3<f32>(0.1 * sin(u.time * 0.4), 0.1 * cos(u.time * 0.6), -7.0);
    let rd = normalize(vec3<f32>(uv, 0.0) - ro);

    var dist = raymarch(ro, rd);

    var color = vec3<f32>(0.0);
    if (dist > 0.0) {
        var p = ro + dist * rd;
        var N = get_normal(ro + dist * rd);
        var L = -rd;
        
        var c0 = vec3<f32>(1.0,0.2,0.3) * N.y;
        color = c0;
        
        var c1 = vec3<f32>(0.4,1.0,1.0) * -N.z;
        color += .5 * c1;
        
        var c2 = vec3<f32>(0.5,0.5,1.0);
        
        var ambient = 0.5 + 0.45 * cos(dist * 7.0);
        color *= 0.7 + ambient * c2;
    }


    return vec4<f32>(color, 1.);
}";


 fn main() {
     env_logger::init(); // Necessary for logging within WGPU
     let event_loop = EventLoop::new(); // Loop provided by winit for handling window events
     let window = WindowBuilder::new()
     .with_title("wgpu-spielerei")
     .with_inner_size(winit::dpi::LogicalSize::new(960.0, 540.0))
     .build(&event_loop).unwrap();
 
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
 
    let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&VS)),
    });

    let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&FS)),
    });

    let mut uniforms = Uniforms { resolution: [size.width.clone() as _, size.height.clone() as _], time: 0., padding: 0. };
    let time = Instant::now();
    let uniforms_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::bytes_of(&uniforms),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      });
    let uniforms_buffer_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: None,
      entries: &[BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::FRAGMENT,
        count: None,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
      }],
    });

    let uniforms_buffer_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &uniforms_buffer_layout,
        entries: &[BindGroupEntry {
          binding: 0,
          resource: uniforms_buffer.as_entire_binding(),
        }],
        });
  
    let render_pipeline_layout =
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniforms_buffer_layout],
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

     // Opens the window and starts processing events
     event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
    
        match event {
            Event::RedrawRequested(_) => {
                uniforms.time = time.elapsed().as_secs_f32();
                queue.write_buffer(&uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));

                let output = surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
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
                      render_pass.set_bind_group(0, &uniforms_buffer_bind_group, &[]);
                      render_pass.draw(0..3, 0..1);
                    }
     
                // submit will accept anything that implements IntoIter
                queue.submit(std::iter::once(encoder.finish()));
                output.present();
              },
            // New
            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                 event: WindowEvent::KeyboardInput { input, .. },
                 window_id,
             } if window_id == window.id() => {
                /*
                * Close on Escape
                */
                 if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                     *control_flow = ControlFlow::Exit
                 }
             }
            _ => (),
        }
    });
 }