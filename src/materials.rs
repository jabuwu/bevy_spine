use std::sync::{Arc, Mutex, Once};

use bevy::{
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

#[derive(Default)]
pub(crate) struct SpineShader {
    vertex: Handle<Shader>,
    fragment: Handle<Shader>,
    fragment_pma: Handle<Shader>,
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

    pub(crate) fn set(
        vertex: Handle<Shader>,
        fragment: Handle<Shader>,
        fragment_pma: Handle<Shader>,
    ) {
        let singleton = SpineShader::singleton();
        let mut shaders = singleton.lock().unwrap();
        shaders.vertex = vertex;
        shaders.fragment = fragment;
        shaders.fragment_pma = fragment_pma;
    }

    pub(crate) fn get_vertex() -> Handle<Shader> {
        SpineShader::singleton().lock().unwrap().vertex.clone()
    }

    pub(crate) fn get_fragment() -> Handle<Shader> {
        SpineShader::singleton().lock().unwrap().fragment.clone()
    }

    pub(crate) fn get_fragment_pma() -> Handle<Shader> {
        SpineShader::singleton()
            .lock()
            .unwrap()
            .fragment_pma
            .clone()
    }
}

macro_rules! material {
    ($uuid:literal, $name:ident, $shader:expr, $blend_state:expr) => {
        #[derive(AsBindGroup, TypeUuid, Clone)]
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
                SpineShader::get_vertex().into()
            }

            fn fragment_shader() -> ShaderRef {
                $shader.into()
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
    };
}

material!(
    "22413663-46b0-4b9b-b714-d72fb87dc7ef",
    SpineNormalMaterial,
    SpineShader::get_fragment(),
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
    "092d3b15-c3b4-45d6-95fd-3a24a86e08d7",
    SpineAdditiveMaterial,
    SpineShader::get_fragment(),
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
    "ec4d2018-ad8f-4ff8-bbf7-33f13dab7ef3",
    SpineMultiplyMaterial,
    SpineShader::get_fragment(),
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
    "5d357844-6a06-4238-aaef-9da95186590b",
    SpineScreenMaterial,
    SpineShader::get_fragment(),
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
    "296e2f58-f5f0-4a51-9f4b-dbcec06ddc04",
    SpineNormalPmaMaterial,
    SpineShader::get_fragment_pma(),
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
    "0f546186-4e05-434b-a0e1-3e1454b2cc7a",
    SpineAdditivePmaMaterial,
    SpineShader::get_fragment_pma(),
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
    "d8ef56cf-88b9-46f8-971b-7583baf8c20b",
    SpineMultiplyPmaMaterial,
    SpineShader::get_fragment_pma(),
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
    "1cd4d391-e106-4585-928f-124f998f28b6",
    SpineScreenPmaMaterial,
    SpineShader::get_fragment_pma(),
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
