#import bevy_sprite::mesh2d_functions as mesh_functions
#import bevy_sprite::mesh2d_bindings       mesh
#import bevy_sprite::mesh2d_vertex_output  MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings  view

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(10) dark_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(10) dark_color: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex.uv;
    out.world_position = mesh_functions::mesh2d_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.clip_position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
    out.world_normal = mesh_functions::mesh2d_normal_local_to_world(vertex.normal);
    out.color = vertex.color;
    out.dark_color = vertex.dark_color;
    return out;
}
