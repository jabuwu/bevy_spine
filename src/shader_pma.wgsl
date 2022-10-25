// The horrible things happening in this shader fixes what *seems* to be a Bevy texture import bug,
// but I haven't been able to narrow down exactly what's going on. Basically, the premultiplied
// alpha step from Spine is happening in one color space, but not being carried over properly to
// Bevy. The solution is to switch back to sRGB color space (linear_to_nonlinear), unpremultiply
// the alpha (unpremultiply), and then switch back to linear space (nonlinear_to_linear), then
// re-multiply the alpha. Seems to kinda defeat the point but here we are. Will need to
// investigate further.

// See: https://github.com/bevyengine/bevy/issues/6315

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct Material {
    color: vec4<f32>,
    dark_color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: Material;
@group(1) @binding(1)
var texture: texture_2d<f32>;
@group(1) @binding(2)
var texture_sampler: sampler;

fn linear_to_nonlinear(x: f32) -> f32 {
    if x <= 0.0 {
        return x;
    }
    if x <= 0.0031308 {
        return x * 12.92;
    } else {
        return (1.055 * pow(x, 1.0 / 2.4)) - 0.055;
    }
}

fn nonlinear_to_linear(x: f32) -> f32 {
    if x <= 0.0 {
        return x;
    }
    if x <= 0.04045 {
        return x / 12.92;
    } else {
        return pow((x + 0.055) / 1.055, 2.4);
    }
}

fn unpremultiply(rgb: vec3<f32>, a: f32) -> vec3<f32> {
    if (a == 0.0) {
        return vec3(0.0, 0.0, 0.0);
    }
    return vec3(rgb.r / a, rgb.g / a, rgb.b / a);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, input.uv);
    let a = tex_color.a;
    let s = vec3(linear_to_nonlinear(tex_color.r), linear_to_nonlinear(tex_color.g), linear_to_nonlinear(tex_color.b));
    let s_non_premult = unpremultiply(s, a);
    let lin = vec3(nonlinear_to_linear(s_non_premult.r) * a, nonlinear_to_linear(s_non_premult.g) * a, nonlinear_to_linear(s_non_premult.b) * a);
    return vec4(
        ((tex_color.a - 1.0) * material.dark_color.a + 1.0 - lin.rgb) * material.dark_color.rgb + lin.rgb * material.color.rgb,
        tex_color.a * material.color.a,
    );
}
