use std::{path::Path, sync::Arc};

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
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
