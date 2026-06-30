use std::{hash::Hash, marker::PhantomData, mem::size_of_val, ops::Range};

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::{RenderAssets, prepare_assets},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{Buffer, BufferDescriptor, BufferUsages, IndexFormat},
        renderer::{RenderDevice, RenderQueue},
        sync_world::{MainEntity, MainEntityHashMap},
        view::{ExtractedView, RenderVisibleEntities},
    },
};
#[cfg(feature = "3d")]
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    material::RenderPhaseType,
    pbr::{
        MATERIAL_BIND_GROUP_INDEX as MATERIAL_3D_BIND_GROUP_INDEX, PreparedMaterial,
        RenderMaterialInstances, SetMaterialBindGroup, SetMeshBindGroup, SetMeshViewBindGroup,
        SetMeshViewBindingArrayBindGroup, queue_material_meshes,
    },
    render::erased_render_asset::ErasedRenderAssets,
};
use bevy_sprite_render::{
    AlphaMode2d, MATERIAL_2D_BIND_GROUP_INDEX, Material2d, PreparedMaterial2d,
    RenderMaterial2dInstances, SetMaterial2dBindGroup, SetMesh2dBindGroup, SetMesh2dViewBindGroup,
    queue_material2d_meshes,
};

use crate::materials::{
    SpineAdditiveMaterial, SpineAdditivePmaMaterial, SpineMultiplyMaterial,
    SpineMultiplyPmaMaterial, SpineNormalMaterial, SpineNormalPmaMaterial, SpineScreenMaterial,
    SpineScreenPmaMaterial,
};

const MIN_BUFFER_CAPACITY: u64 = 4096;

/// CPU-side Spine geometry extracted to the render world for direct GPU upload.
#[derive(Component, Clone, Default, ExtractComponent)]
pub(crate) struct SpineDirectMesh {
    vertices: Vec<SpineDirectVertex>,
    indices: Vec<u16>,
}

impl SpineDirectMesh {
    pub(crate) fn write(
        &mut self,
        mesh_entity: Entity,
        vertices: &[[f32; 2]],
        indices: &[u16],
        uvs: &[[f32; 2]],
        colors: &[[f32; 4]],
        dark_colors: &[[f32; 4]],
    ) -> bool {
        let vertex_count = vertices.len();
        if uvs.len() != vertex_count {
            warn!(
                entity = ?mesh_entity,
                "Spine renderable has {vertex_count} vertices but {} uvs; skipping direct mesh update",
                uvs.len()
            );
            self.clear();
            return false;
        }
        if colors.len() != vertex_count || dark_colors.len() != vertex_count {
            warn!(
                entity = ?mesh_entity,
                "Spine renderable has {vertex_count} vertices but {} colors and {} dark colors; skipping direct mesh update",
                colors.len(),
                dark_colors.len()
            );
            self.clear();
            return false;
        }
        if indices
            .iter()
            .any(|index| usize::from(*index) >= vertex_count)
        {
            warn!(
                entity = ?mesh_entity,
                "Spine renderable has an index outside its {vertex_count} vertices; skipping direct mesh update"
            );
            self.clear();
            return false;
        }

        self.vertices.clear();
        self.vertices
            .extend(vertices.iter().zip(uvs).zip(colors).zip(dark_colors).map(
                |(((position, uv), color), dark_color)| SpineDirectVertex {
                    position: [position[0], position[1], 0.0],
                    normal: [0.0, 0.0, 0.0],
                    uv: *uv,
                    color: *color,
                    dark_color: *dark_color,
                },
            ));

        self.indices.clear();
        self.indices.extend_from_slice(indices);
        true
    }

    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpineDirectVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
    dark_color: [f32; 4],
}

#[derive(Resource, Default)]
struct SpineDirectMeshBuffers {
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    vertex_capacity: u64,
    index_capacity: u64,
    meshes: MainEntityHashMap<PackedSpineDirectMesh>,
    vertices: Vec<SpineDirectVertex>,
    indices: Vec<u16>,
}

struct PackedSpineDirectMesh {
    indices: Range<u32>,
    vertex_base: i32,
}

/// Direct-render support for built-in Spine materials.
pub(crate) struct SpineDirectRenderPlugin;

impl Plugin for SpineDirectRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SpineDirectMaterial2dPlugin::<SpineNormalMaterial>::default(),
            SpineDirectMaterial2dPlugin::<SpineAdditiveMaterial>::default(),
            SpineDirectMaterial2dPlugin::<SpineMultiplyMaterial>::default(),
            SpineDirectMaterial2dPlugin::<SpineScreenMaterial>::default(),
            SpineDirectMaterial2dPlugin::<SpineNormalPmaMaterial>::default(),
            SpineDirectMaterial2dPlugin::<SpineAdditivePmaMaterial>::default(),
            SpineDirectMaterial2dPlugin::<SpineMultiplyPmaMaterial>::default(),
            SpineDirectMaterial2dPlugin::<SpineScreenPmaMaterial>::default(),
        ));

        app.add_plugins(ExtractComponentPlugin::<SpineDirectMesh>::extract_visible());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SpineDirectMeshBuffers>()
                .add_systems(
                    Render,
                    prepare_spine_direct_mesh_buffers.in_set(RenderSystems::PrepareResources),
                );

            #[cfg(feature = "3d")]
            render_app
                .add_render_command::<Transparent3d, DrawSpineDirectMaterial3d>()
                .add_systems(
                    Render,
                    queue_spine_direct_material3d_meshes
                        .in_set(RenderSystems::QueueMeshes)
                        .after(queue_material_meshes),
                );
        }
    }
}

/// Queues direct-rendered Spine meshes that use alpha-blended material `M`.
///
/// Built-in Spine materials are registered by [`SpinePlugin`](crate::SpinePlugin). Custom 2D
/// materials also need Bevy's [`Material2dPlugin`](bevy::sprite_render::Material2dPlugin) and
/// [`SpineMaterialPlugin`](crate::materials::SpineMaterialPlugin).
#[derive(Default)]
pub struct SpineDirectMaterial2dPlugin<M: Material2d>(PhantomData<M>);

impl<M> Plugin for SpineDirectMaterial2dPlugin<M>
where
    M: Material2d,
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawSpineDirectMaterial2d<M>>()
                .add_systems(
                    Render,
                    queue_spine_direct_material2d_meshes::<M>
                        .in_set(RenderSystems::QueueMeshes)
                        .after(prepare_assets::<PreparedMaterial2d<M>>)
                        .after(queue_material2d_meshes::<M>),
                );
        }
    }
}

type DrawSpineDirectMaterial2d<M> = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetMaterial2dBindGroup<M, MATERIAL_2D_BIND_GROUP_INDEX>,
    DrawSpineDirectMesh,
);

#[cfg(feature = "3d")]
type DrawSpineDirectMaterial3d = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetMeshBindGroup<2>,
    SetMaterialBindGroup<MATERIAL_3D_BIND_GROUP_INDEX>,
    DrawSpineDirectMesh,
);

fn prepare_spine_direct_mesh_buffers(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut buffers: ResMut<SpineDirectMeshBuffers>,
    meshes: Query<(&MainEntity, &SpineDirectMesh)>,
) {
    buffers.prepare(&render_device, &render_queue, meshes.iter());
}

impl SpineDirectMeshBuffers {
    fn prepare<'a>(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        meshes: impl IntoIterator<Item = (&'a MainEntity, &'a SpineDirectMesh)>,
    ) {
        self.meshes.clear();
        self.vertices.clear();
        self.indices.clear();

        for (main_entity, mesh) in meshes {
            self.pack(*main_entity, mesh);
        }

        let vertex_size = size_of_val(self.vertices.as_slice()) as u64;
        let index_size = size_of_val(self.indices.as_slice()) as u64;

        if vertex_size > self.vertex_capacity {
            self.vertex_capacity = next_buffer_capacity(vertex_size);
            self.vertex_buffer = Some(render_device.create_buffer(&BufferDescriptor {
                label: Some("spine_direct_packed_vertex_buffer"),
                size: self.vertex_capacity,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if index_size > self.index_capacity {
            self.index_capacity = next_buffer_capacity(index_size);
            self.index_buffer = Some(render_device.create_buffer(&BufferDescriptor {
                label: Some("spine_direct_packed_index_buffer"),
                size: self.index_capacity,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if vertex_size > 0 {
            let Some(buffer) = &self.vertex_buffer else {
                error!("missing Spine direct vertex buffer after capacity check; skipping upload");
                return;
            };
            render_queue.write_buffer(buffer, 0, bytemuck::cast_slice(self.vertices.as_slice()));
        }

        if index_size > 0 {
            let Some(buffer) = &self.index_buffer else {
                error!("missing Spine direct index buffer after capacity check; skipping upload");
                return;
            };
            render_queue.write_buffer(buffer, 0, bytemuck::cast_slice(self.indices.as_slice()));
        }
    }

    fn pack(&mut self, main_entity: MainEntity, mesh: &SpineDirectMesh) {
        if mesh.vertices.is_empty() || mesh.indices.is_empty() {
            return;
        }

        let Some(vertex_base) = i32::try_from(self.vertices.len()).ok() else {
            error!(entity = ?main_entity, "too many packed Spine vertices; skipping mesh");
            return;
        };
        let Some(index_start) = u32::try_from(self.indices.len()).ok() else {
            error!(entity = ?main_entity, "too many packed Spine indices; skipping mesh");
            return;
        };
        let Some(index_count) = u32::try_from(mesh.indices.len()).ok() else {
            error!(entity = ?main_entity, "Spine mesh has too many indices; skipping mesh");
            return;
        };
        let Some(index_end) = index_start.checked_add(index_count) else {
            error!(entity = ?main_entity, "too many packed Spine indices; skipping mesh");
            return;
        };

        self.vertices.extend_from_slice(&mesh.vertices);
        self.indices.extend_from_slice(&mesh.indices);
        self.meshes.insert(
            main_entity,
            PackedSpineDirectMesh {
                indices: index_start..index_end,
                vertex_base,
            },
        );
    }
}

fn next_buffer_capacity(required: u64) -> u64 {
    required
        .max(MIN_BUFFER_CAPACITY)
        .checked_next_power_of_two()
        .unwrap_or(required)
}

fn queue_spine_direct_material2d_meshes<M: Material2d>(
    render_materials: Res<RenderAssets<PreparedMaterial2d<M>>>,
    render_material_instances: Res<RenderMaterial2dInstances<M>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    spine_meshes: Query<&SpineDirectMesh>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    let draw_function = draw_functions.read().id::<DrawSpineDirectMaterial2d<M>>();

    for (view, view_visible_entities) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let Some(visible_entities) = view_visible_entities.get::<Mesh2d>() else {
            continue;
        };

        for (_, main_entity) in &visible_entities.removed_entities {
            transparent_phase.remove(Entity::PLACEHOLDER, *main_entity);
        }

        for (render_entity, visible_entity) in visible_entities.iter_visible() {
            if !spine_meshes.contains(*render_entity) {
                continue;
            }
            let Some(material_asset_id) = render_material_instances.get(visible_entity) else {
                // Another registered direct material queue owns this mesh.
                continue;
            };

            let Some(material_2d) = render_materials.get(*material_asset_id) else {
                continue;
            };
            if material_2d.properties.alpha_mode != AlphaMode2d::Blend {
                transparent_phase.remove(Entity::PLACEHOLDER, *visible_entity);
                warn!(
                    entity = ?visible_entity,
                    material = std::any::type_name::<M>(),
                    "Spine direct 2D rendering requires an alpha-blended Material2d; skipping mesh"
                );
                continue;
            }
            let Some(item) = transparent_phase
                .items
                .get_mut(&(Entity::PLACEHOLDER, *visible_entity))
            else {
                continue;
            };
            item.draw_function = draw_function;
            item.batch_range = 0..1;
            item.extra_index = PhaseItemExtraIndex::None;
            item.indexed = true;
        }
    }
}

#[cfg(feature = "3d")]
fn queue_spine_direct_material3d_meshes(
    render_materials: Option<Res<ErasedRenderAssets<PreparedMaterial>>>,
    render_material_instances: Option<Res<RenderMaterialInstances>>,
    transparent_render_phases: Option<ResMut<ViewSortedRenderPhases<Transparent3d>>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    spine_meshes: Query<&SpineDirectMesh>,
    draw_functions: Res<DrawFunctions<Transparent3d>>,
) {
    let Some(render_materials) = render_materials else {
        return;
    };
    let Some(render_material_instances) = render_material_instances else {
        return;
    };
    let Some(mut transparent_render_phases) = transparent_render_phases else {
        return;
    };
    let draw_function = draw_functions.read().id::<DrawSpineDirectMaterial3d>();

    for (view, view_visible_entities) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let Some(visible_entities) = view_visible_entities.get::<Mesh3d>() else {
            continue;
        };

        for (_, main_entity) in &visible_entities.removed_entities {
            transparent_phase.remove(Entity::PLACEHOLDER, *main_entity);
        }

        for (render_entity, visible_entity) in visible_entities.iter_visible() {
            if !spine_meshes.contains(*render_entity) {
                continue;
            }
            let Some(material_instance) = render_material_instances.instances.get(visible_entity)
            else {
                continue;
            };
            let Some(material) = render_materials.get(material_instance.asset_id) else {
                continue;
            };

            if !matches!(
                material.properties.render_phase_type,
                RenderPhaseType::Transparent
            ) {
                transparent_phase.remove(Entity::PLACEHOLDER, *visible_entity);
                warn!(
                    entity = ?visible_entity,
                    "Spine direct 3D rendering requires an alpha-blended Material; skipping mesh"
                );
                continue;
            }

            let Some(item) = transparent_phase
                .items
                .get_mut(&(Entity::PLACEHOLDER, *visible_entity))
            else {
                continue;
            };
            item.draw_function = draw_function;
            item.batch_range = 0..1;
            item.extra_index = PhaseItemExtraIndex::None;
            item.indexed = true;
        }
    }
}

struct DrawSpineDirectMesh;

impl<P: bevy::render::render_phase::PhaseItem> RenderCommand<P> for DrawSpineDirectMesh {
    type Param = bevy::ecs::system::lifetimeless::SRes<SpineDirectMeshBuffers>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        _view: (),
        _entity: Option<()>,
        buffers: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let buffers = buffers.into_inner();
        let Some(mesh) = buffers.meshes.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        let (Some(vertex_buffer), Some(index_buffer)) =
            (&buffers.vertex_buffer, &buffers.index_buffer)
        else {
            return RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
        pass.draw_indexed(
            mesh.indices.clone(),
            mesh.vertex_base,
            item.batch_range().clone(),
        );

        RenderCommandResult::Success
    }
}
