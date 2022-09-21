use std::{path::Path, sync::Arc};

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::Handle,
    reflect::TypeUuid,
    utils::BoxedFuture,
};

#[derive(Debug, TypeUuid)]
#[uuid = "e58e872a-9d35-41bf-b561-95f843686004"]
pub struct Atlas {
    pub atlas: Arc<rusty_spine::Atlas>,
}

#[derive(Default)]
pub struct AtlasLoader;

impl AssetLoader for AtlasLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            match rusty_spine::Atlas::new(
                bytes,
                load_context.path().parent().unwrap_or(Path::new("")),
            ) {
                Ok(atlas) => {
                    load_context.set_default_asset(LoadedAsset::new(Atlas {
                        atlas: Arc::new(atlas),
                    }));
                    Ok(())
                }
                Err(_) => Err(bevy::asset::Error::msg(format!(
                    "Failed to load Spine atlas: {:?}",
                    load_context.path()
                ))),
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["atlas"]
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "8637cf16-90c4-4825-bdf2-277e38788365"]
pub struct SkeletonJson {
    pub json: Vec<u8>,
}

#[derive(Default)]
pub struct SkeletonJsonLoader;

impl AssetLoader for SkeletonJsonLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(SkeletonJson {
                json: bytes.to_vec(),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "2a2a342a-29ae-4417-adf5-06ea7f0732d0"]
pub struct SkeletonBinary {
    pub binary: Vec<u8>,
}

#[derive(Default)]
pub struct SkeletonBinaryLoader;

impl AssetLoader for SkeletonBinaryLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(SkeletonBinary {
                binary: bytes.to_vec(),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["skel"]
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "7796a37b-37a4-49ea-bf4e-fb7344aa6015"]
pub struct SkeletonData {
    pub load_type: SkeletonDataTypeHandle,
    pub atlas: Handle<Atlas>,
    pub loader: Option<SkeletonDataLoader>,
    pub data: Option<Arc<rusty_spine::SkeletonData>>,
}

#[derive(Debug)]
pub enum SkeletonDataTypeHandle {
    Json(Handle<SkeletonJson>),
    Binary(Handle<SkeletonBinary>),
}

#[derive(Debug)]
pub enum SkeletonDataLoader {
    Json(rusty_spine::SkeletonJson),
    Binary(rusty_spine::SkeletonBinary),
}

impl SkeletonData {
    pub fn new_from_json(json: Handle<SkeletonJson>, atlas: Handle<Atlas>) -> Self {
        Self {
            load_type: SkeletonDataTypeHandle::Json(json),
            atlas,
            loader: None,
            data: None,
        }
    }
    pub fn new_from_binary(binary: Handle<SkeletonBinary>, atlas: Handle<Atlas>) -> Self {
        Self {
            load_type: SkeletonDataTypeHandle::Binary(binary),
            atlas,
            loader: None,
            data: None,
        }
    }
}
