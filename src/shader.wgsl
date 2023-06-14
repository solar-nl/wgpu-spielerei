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
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    resolution : vec2<f32>,
    time : f32
};

let epsilon = 0.0001;
let pi = 3.1415926539;


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

fn rotate2D(plane: vec2<f32>, angle: f32) -> vec2<f32> {
    return cos(angle) * plane + sin(angle) * vec2<f32>(plane.y, -plane.x);
}

fn sdTriangleIsosceles(pos: vec2<f32>, q: vec2<f32>) -> f32 {
    var p = pos;
    p.x = abs(p.x);
    var a: vec2<f32> = p - q * clamp(dot(p, q) / dot(q, q), 0.0, 1.0);
    var b: vec2<f32> = p - q * vec2(clamp(p.x / q.x, 0.0, 1.0), 1.0);
    var s: f32 = -sign(q.y);
    var d: vec2<f32> = min(vec2<f32>(dot(a, a), s * (p.x * q.y - p.y * q.x)),
                      vec2<f32>(dot(b, b), s * (p.y - q.y)));
    return -sqrt(d.x) * sign(d.y);
}

fn solar_logo(pos: vec2<f32>) -> f32 {
    var p = rotate2D(pos, -u.time * 0.18);

    var outer = length(pos) - 0.3;
    var inner = length(pos) - 0.24;
    let ring = max(outer, -inner);   

    var tri = 10.0;
    let offset = vec2<f32>(0.0, 0.447);
 
    let angle = length(p + offset);
    for (var i: i32 = 0; i < 8; i++) {
        p = rotate2D(p, 8.0 * pi / 32.0);
        tri = min(tri, sdTriangleIsosceles(rotate2D(p + offset, length(p - vec2<f32>(0.2))), vec2<f32>(0.02, 0.23)));
    }
    return min(tri, ring);
    
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
        
        var c2 = vec3<f32>(0.25,0.25,1.0);
        
        var ambient = 0.5 + 0.45 * cos(dist * 7.0);
        color *= 0.7 + ambient * c2;
    }

    var logo = solar_logo(uv * 1.5);
    if (logo < 0.0) {
        color += vec3<f32>(0.5, 0.5, 0.5);
    }
    return vec4<f32>(color, 1.);
}