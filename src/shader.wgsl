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

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, input.uv);
    return vec4(
        ((tex_color.a - 1.0) * material.dark_color.a + 1.0 - tex_color.rgb) * material.dark_color.rgb + tex_color.rgb * material.color.rgb,
        tex_color.a * material.color.a,
    );
}
