//! Materials for Spine meshes.
//!
//! To create a custom material for Spine, see [`SpineMaterial`].

use std::marker::PhantomData;

use bevy::{
    asset::Asset,
    ecs::system::{StaticSystemParam, SystemParam},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
            RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, VertexAttribute,
            VertexFormat,
        },
    },
    sprite::{Material2d, Material2dKey},
};
use rusty_spine::BlendMode;

use crate::{Spine, SpineMesh, SpineMeshState, SpineSettings, SpineSystem};

/// Trait for automatically applying materials to [`SpineMesh`] entities. Used by the built-in
/// materials but can also be used to create custom materials.
///
/// Implement the trait and add it with [`SpineMaterialPlugin`].
pub trait SpineMaterial: Sized {
    /// The material type to apply to [`SpineMesh`]. Usually is `Self`.
    type Material: Asset;
    /// System parameters to query when updating this material.
    type Params<'w, 's>: SystemParam;

    /// Ran every frame for every material and every [`SpineMesh`].
    ///
    /// If this function returns [`Some`], then the material will be applied to the [`SpineMesh`],
    /// otherwise it will be removed. Default materials should be removed if a custom material is
    /// desired (see [`SpineSettings::default_materials`]).
    fn update<'w, 's>(
        material: Option<Self::Material>,
        entity: Entity,
        renderable_data: SpineMaterialInfo,
        params: &StaticSystemParam<Self::Params<'w, 's>>,
    ) -> Option<Self::Material>;
}

/// Add support for a new [`SpineMaterial`].
pub struct SpineMaterialPlugin<T: SpineMaterial> {
    _marker: PhantomData<T>,
}

impl<T: SpineMaterial> Default for SpineMaterialPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: SpineMaterial + Send + Sync + 'static> Plugin for SpineMaterialPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_materials::<T>
                .in_set(SpineSystem::UpdateMaterials)
                .after(SpineSystem::UpdateMeshes),
        );
    }
}

/// Info necessary for a Spine material.
#[derive(Clone)]
pub struct SpineMaterialInfo {
    pub texture: Handle<Image>,
    pub blend_mode: BlendMode,
    pub premultiplied_alpha: bool,
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn update_materials<'w, 's, T: SpineMaterial>(
    mut commands: Commands,
    mut materials: ResMut<Assets<T::Material>>,
    spine_query: Query<(Entity, &Children), With<Spine>>,
    mesh_query: Query<(Entity, &SpineMesh, Option<&Handle<T::Material>>)>,
    params: StaticSystemParam<T::Params<'w, 's>>,
) {
    for (spine_entity, spine_children) in spine_query.iter() {
        for spine_child in spine_children.iter() {
            if let Ok((mesh_entity, spine_mesh, material_handle)) = mesh_query.get(*spine_child) {
                let SpineMeshState::Renderable { info: data } = spine_mesh.state.clone() else {
                    continue;
                };
                if let Some(handle) = material_handle {
                    if let Some(material) =
                        T::update(materials.remove(handle), spine_entity, data, &params)
                    {
                        materials.set_untracked(handle, material);
                    }
                } else {
                    if let Some(material) = T::update(None, spine_entity, data, &params) {
                        let handle = materials.add(material);
                        #[cfg(feature = "workaround_5732")]
                        {
                            crate::workaround_5732::store(handle.clone_untyped());
                        }
                        if let Some(mut entity_commands) = commands.get_entity(mesh_entity) {
                            entity_commands.insert(handle.clone());
                        }
                    }
                };
            }
        }
    }
}

pub const VERTEX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10655547040990968849);
pub const FRAGMENT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10048211129967055809);

/// A [`SystemParam`] to query [`SpineSettings`].
///
/// Mostly used for the built-in materials but may be useful for implementing other materials.
#[derive(SystemParam)]
pub struct SpineSettingsQuery<'w, 's> {
    pub spine_settings_query: Query<'w, 's, &'static SpineSettings>,
}

macro_rules! material {
    ($(#[$($attrss:tt)*])* $uuid:literal, $name:ident, $blend_mode:expr, $premultiplied_alpha:expr, $blend_state:expr) => {
        $(#[$($attrss)*])*
        #[derive(Default, AsBindGroup, TypeUuid, Clone)]
        #[uuid = $uuid]
        pub struct $name {
            #[texture(0)]
            #[sampler(1)]
            pub image: Handle<Image>,
        }

        impl $name {
            pub fn new(image: Handle<Image>) -> Self {
                Self { image }
            }
        }

        impl Material2d for $name {
            fn vertex_shader() -> ShaderRef {
                VERTEX_SHADER_HANDLE.typed().into()
            }

            fn fragment_shader() -> ShaderRef {
                FRAGMENT_SHADER_HANDLE.typed().into()
            }

            fn specialize(
                descriptor: &mut RenderPipelineDescriptor,
                _layout: &MeshVertexBufferLayout,
                _key: Material2dKey<Self>,
            ) -> Result<(), SpecializedMeshPipelineError> {
                descriptor.vertex.buffers[0]
                    .attributes
                    .push(VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 44,
                        shader_location: 5,
                    });
                if let Some(fragment) = &mut descriptor.fragment {
                    if let Some(target_state) = &mut fragment.targets[0] {
                        target_state.blend = Some($blend_state);
                    }
                }
                descriptor.primitive.cull_mode = None;
                Ok(())
            }
        }

        impl SpineMaterial for $name {
            type Material = Self;
            type Params<'w, 's> = SpineSettingsQuery<'w, 's>;

            fn update<'w, 's>(
                material: Option<Self>,
                entity: Entity,
                renderable_data: SpineMaterialInfo,
                params: &StaticSystemParam<Self::Params<'w, 's>>,
            ) -> Option<Self> {
                let spine_settings = params.spine_settings_query.get(entity).copied().unwrap_or(SpineSettings::default());
                if spine_settings.default_materials && renderable_data.blend_mode == $blend_mode && renderable_data.premultiplied_alpha == $premultiplied_alpha {
                    let mut material = material.unwrap_or_else(|| Self::default());
                    material.image = renderable_data.texture;
                    Some(material)
                } else {
                    None
                }
            }
        }
    };
}

material!(
    /// Normal blend mode material, non-premultiplied-alpha
    "22413663-46b0-4b9b-b714-d72fb87dc7ef",
    SpineNormalMaterial,
    BlendMode::Normal,
    false,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    }
);

material!(
    /// Additive blend mode material, non-premultiplied-alpha
    "092d3b15-c3b4-45d6-95fd-3a24a86e08d7",
    SpineAdditiveMaterial,
    BlendMode::Additive,
    false,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    }
);

material!(
    /// Multiply blend mode material, non-premultiplied-alpha
    "ec4d2018-ad8f-4ff8-bbf7-33f13dab7ef3",
    SpineMultiplyMaterial,
    BlendMode::Multiply,
    false,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::Dst,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusSrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    }
);

material!(
    /// Screen blend mode material, non-premultiplied-alpha
    "5d357844-6a06-4238-aaef-9da95186590b",
    SpineScreenMaterial,
    BlendMode::Screen,
    false,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusSrc,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    }
);

material!(
    /// Normal blend mode material, premultiplied-alpha
    "296e2f58-f5f0-4a51-9f4b-dbcec06ddc04",
    SpineNormalPmaMaterial,
    BlendMode::Normal,
    true,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    }
);

material!(
    /// Additive blend mode material, premultiplied-alpha
    "0f546186-4e05-434b-a0e1-3e1454b2cc7a",
    SpineAdditivePmaMaterial,
    BlendMode::Additive,
    true,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    }
);

material!(
    /// Multiply blend mode material, premultiplied-alpha
    "d8ef56cf-88b9-46f8-971b-7583baf8c20b",
    SpineMultiplyPmaMaterial,
    BlendMode::Multiply,
    true,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::Dst,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusSrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    }
);

material!(
    /// Screen blend mode material, premultiplied-alpha
    "1cd4d391-e106-4585-928f-124f998f28b6",
    SpineScreenPmaMaterial,
    BlendMode::Screen,
    true,
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::OneMinusSrc,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    }
);
