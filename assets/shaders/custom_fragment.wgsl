struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) dark_color: vec4<f32>,
};

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var<uniform> time: f32;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let time_sin = 0.5 + sin(time * 10.0) * 0.5;
    let tex_color = vec4(
        textureSample(texture, texture_sampler, input.uv).rgb * time_sin + (1.0 - textureSample(texture, texture_sampler, input.uv).rgb) * (1.0 - time_sin),
        textureSample(texture, texture_sampler, input.uv).a
    );
    return vec4(
        ((tex_color.a - 1.0) * input.dark_color.a + 1.0 - tex_color.rgb) * input.dark_color.rgb + tex_color.rgb * input.color.rgb,
        tex_color.a * input.color.a,
    );
}
