use std::sync::{Arc, Mutex, Once};

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
            RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey},
};

#[derive(Default)]
pub(crate) struct SpineShader {
    handle: Handle<Shader>,
    handle_pma: Handle<Shader>,
}

impl SpineShader {
    fn singleton() -> Arc<Mutex<SpineShader>> {
        static START: Once = Once::new();
        static mut INSTANCE: Option<Arc<Mutex<SpineShader>>> = None;
        START.call_once(|| unsafe {
            INSTANCE = Some(Arc::new(Mutex::new(SpineShader::default())));
        });
        unsafe {
            let singleton = INSTANCE.as_ref().unwrap();
            singleton.clone()
        }
    }

    pub(crate) fn set(handle: Handle<Shader>, handle_pma: Handle<Shader>) {
        SpineShader::singleton().lock().unwrap().handle = handle;
        SpineShader::singleton().lock().unwrap().handle_pma = handle_pma;
    }

    pub(crate) fn get() -> Handle<Shader> {
        SpineShader::singleton().lock().unwrap().handle.clone()
    }

    pub(crate) fn get_pma() -> Handle<Shader> {
        SpineShader::singleton().lock().unwrap().handle_pma.clone()
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "22413663-46b0-4b9b-b714-d72fb87dc7ef"]
pub struct SpineNormalMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineNormalMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineNormalMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "092d3b15-c3b4-45d6-95fd-3a24a86e08d7"]
pub struct SpineAdditiveMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineAdditiveMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineAdditiveMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "ec4d2018-ad8f-4ff8-bbf7-33f13dab7ef3"]
pub struct SpineMultiplyMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineMultiplyMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineMultiplyMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "5d357844-6a06-4238-aaef-9da95186590b"]
pub struct SpineScreenMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineScreenMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineScreenMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "296e2f58-f5f0-4a51-9f4b-dbcec06ddc04"]
pub struct SpineNormalPmaMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineNormalPmaMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineNormalPmaMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get_pma().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "0f546186-4e05-434b-a0e1-3e1454b2cc7a"]
pub struct SpineAdditivePmaMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineAdditivePmaMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineAdditivePmaMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get_pma().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "d8ef56cf-88b9-46f8-971b-7583baf8c20b"]
pub struct SpineMultiplyPmaMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineMultiplyPmaMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineMultiplyPmaMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get_pma().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "1cd4d391-e106-4585-928f-124f998f28b6"]
pub struct SpineScreenPmaMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub dark_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub image: Handle<Image>,
}

impl SpineScreenPmaMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            color: Color::NONE,
            dark_color: Color::BLACK,
            image,
        }
    }
}

impl Material2d for SpineScreenPmaMaterial {
    fn fragment_shader() -> ShaderRef {
        SpineShader::get_pma().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState {
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
                });
            }
        }
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
