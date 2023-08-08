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
    time : f32,
    i_pass : i32,
};

let epsilon = 0.0001;
let pi = 3.1415926539;
let iterations = 40;

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var samp : sampler;
@group(0) @binding(2) var tex_0: texture_2d<f32>;
@group(0) @binding(3) var tex_1: texture_2d<f32>;
@group(0) @binding(4) var tex_2: texture_2d<f32>;
@group(0) @binding(5) var tex_3: texture_2d<f32>;


fn rotate2D(plane: vec2<f32>, angle: f32) -> vec2<f32> {
    return cos(angle) * plane + sin(angle) * vec2(plane.y,-plane.x);
}

fn rotate3D(p: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
	var a = cross(axis, p);        
    var b = cross(a, axis);
    
	return b * cos(angle) + a * sin(angle) + axis * dot(p, axis);   
}

fn sdBox(pos: vec3<f32>, b: vec3<f32>) -> f32 {
    var d = abs(pos) - b;
    return min(max(d.x, max(d.y, d.z)), 0.0) + length(max(d, vec3<f32>(0.0)));
}

fn maxcomp(p: vec2<f32>) -> f32{
	return max(p.x, p.y);
}

fn sdCross(p: vec3<f32>) -> f32 {
  let da = maxcomp(abs(p.xy));
  let db = maxcomp(abs(p.yz));
  let dc = maxcomp(abs(p.zx));
  return min(da, min(db, dc)) - 1.0;
}

fn not_menger(pos: vec3<f32>) -> f32 {
    
    var q = pos;
 /*   
    q = rotate3D(q, vec3<f32>(0.0, 1.0, 0.0), u.time * 0.7);
    q = rotate3D(q, vec3<f32>(1.0, 0.0, 0.0), u.time * 0.5);
*/
    let scale = 6.0;
    let iteration = 1.0;
    let offset = 0.02;

    var p = abs(fract(q / scale) * scale - scale * 0.5);    
 	var d = sdCross(p) + offset;
    for (var i = 0.0; i < iteration; i = i + 1.0) {
        p = abs(fract(q * (i / scale + 1.0)) * 0.5 - 1.0 / iteration);
 	    d = max(d, sdCross(p) + 1.0 - 1.0 / (scale * i) + offset);
    }

    return d;
}

fn get_normal(p: vec3<f32>) -> vec3<f32> {
	var dist = not_menger(p);
	return normalize(vec3<f32>(not_menger(p + vec3<f32>(epsilon, 0.0, 0.0)) - dist,
                          not_menger(p + vec3<f32>(0.0, epsilon, 0.0)) - dist,
                          not_menger(p + vec3<f32>(0.0, 0.0, epsilon)) - dist));
}

fn raymarch(origin: vec3<f32>, direction: vec3<f32>) -> vec2<f32> {
    var t = 0.0;
    var near_miss = 0.0;

    for(var i: i32 = 0; i < iterations; i++) {
    	let pos = vec3<f32>(origin + t * direction);
        var dist = not_menger(pos);

        near_miss += t * 0.002;
        t += dist;

        if (dist < epsilon) {
            return vec2<f32>(t - epsilon, near_miss);
        }
      }

    return vec2<f32>(0.0);
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

fn rand(n: f32) -> f32 {return fract(sin(n) * 43758.5453123);}
fn my_mod(x: f32, y: f32) -> f32 {return x - y * floor(x / y);}
fn vhs(tex: texture_2d<f32>, samp: sampler, coords: vec2<f32>, amount: f32, spd: f32, t: f32) -> vec4<f32> { 
    var scale = amount * 0.7314;
    var uv = coords;
    var inner = t * 33.14 * cos(spd * 6.28 + 0.4 * rand(scale));
    var speed = floor(my_mod(inner, 128.0));
    
    var ln = 1.628*rand(speed * 0.5678) * 1.0;
    var width = 1.33*rand(speed+t) * 0.25 * 1.0;    
    var offset = 0.0;
/*
    uv.x += -0.00628 + 0.009*fract(ln);
    uv.y -= -0.00428 + 0.008*fract(ln*ln);
*/
    var color: vec3<f32> = textureSample(tex, samp, uv).rgb;
    var abberated = 0.00314*length(uv)+0.31415*abs(offset);
    
    color.r = textureSample(tex, samp, uv + abberated).r;  
    color.g = textureSample(tex, samp, uv).g;
    color.b = textureSample(tex, samp, uv - abberated).b;  

    return vec4<f32>(color,length(color));
}
/*
fn overlay_texcoords(coords: vec4) -> vec2 {

}
*/
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = (in.tex_coords.xy * u.resolution.xy * 2.0 - u.resolution.xy) / u.resolution.x;
    uv = (1.0 - fract(u.time * 1.5) * 0.2) * rotate2D(uv, 0.1 * pow(fract(u.time * 0.5), 10.0) * 20.1);
    var color = vec3<f32>(0.0);
    if (u.i_pass == 0) {
        let pulse = 5.0 * -u.time;//5.0 * sin(0.5 * u.time);
        var campos = vec3<f32>(0.1 * sin(u.time), 0.12 * cos(u.time * 0.7), u.time * 3.5);
        var camdir = vec3<f32>(0.0, 0.0, 1.0);
        let ratio = u.resolution.y / u.resolution.x;
        let u = vec3<f32>(1.0, 0.0, 0.0);
        var v = vec3<f32>(0.0, 1.0, 0.0);

        var ro = campos;
        var rd = normalize(camdir + uv.x * u + uv.y * v);

        var dist = raymarch(ro, rd);

        if (dist.x > 0.0) {
            var p = ro + dist.x * rd;
            var N = get_normal(p);
            var L = -rd;
            
            dist.y = max(0.17, dist.y);

            var c0 = vec3<f32>(0.7,0.2,0.3) * N.y;
            color = c0;
            
            var c1 = vec3<f32>(0.2,0.1,0.3) * -N.z;
            color += c1;
            color = -color + dist.y;
            color *= color;
            
            var sheen = vec3<f32>(dist.y * 10.0) * dist.y * dist.y * 100.0;
            color *= sheen;
            color = mix(sheen * 0.8, color, 0.7 + 0.3 * pow(2.0, sin(dist.x * 0.5 + pulse)));
        }
    
        var logo = solar_logo(uv * 1.0);
        if (logo < 0.0) {
            color = max(color, vec3<f32>(0.5));
        }

    }


    else if (u.i_pass == 1) {
        let tex_color: vec4<f32> = vhs(tex_0, samp, vec2<f32>(1.0, -1.0) * in.tex_coords.xy, 0.1, 3.0, u.time);

        //let logo: vec4<f32> = textureSample(tex_3, samp, vec2<f32>(1.0, -0.75) * in.tex_coords.xy - vec2<f32>(0.0, 0.15));

        color = tex_color.rgb;// + logo.rgb;
    }

    if (in.tex_coords.y < 0.01 || in.tex_coords.y > 0.99 || in.tex_coords.x < 0.01 || in.tex_coords.x > 0.99) {
        color = vec3<f32>(0.0);
    }

    return vec4<f32>(color, 1.);
}