use std::{path::Path, sync::Arc};

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::BoxedFuture,
};

/// Bevy asset for [`rusty_spine::Atlas`], loaded from `.atlas` files.
///
/// For loading a complete skeleton, see [`SkeletonData`].
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "e58e872a-9d35-41bf-b561-95f843686004"]
pub struct Atlas {
    pub atlas: Arc<rusty_spine::Atlas>,
}

#[derive(Default)]
pub(crate) struct AtlasLoader;

impl AssetLoader for AtlasLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            match rusty_spine::Atlas::new(
                &bytes,
                load_context
                    .path()
                    .parent()
                    .unwrap_or_else(|| Path::new("")),
            ) {
                Ok(atlas) => {
                    load_context.set_default_asset(LoadedAsset::new(Atlas {
                        atlas: Arc::new(atlas),
                    }));
                    Ok(())
                }
                Err(_) => Err(anyhow::Error::msg(format!(
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

/// Bevy asset for [`rusty_spine::SkeletonJson`], loaded from `.json` files.
///
/// For loading a complete skeleton, see [`SkeletonData`].
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "8637cf16-90c4-4825-bdf2-277e38788365"]
pub struct SkeletonJson {
    pub json: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct SkeletonJsonLoader;

impl AssetLoader for SkeletonJsonLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
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

/// Bevy asset for [`rusty_spine::SkeletonBinary`], loaded from `.skel` files.
///
/// For loading a complete skeleton, see [`SkeletonData`].
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "2a2a342a-29ae-4417-adf5-06ea7f0732d0"]
pub struct SkeletonBinary {
    pub binary: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct SkeletonBinaryLoader;

impl AssetLoader for SkeletonBinaryLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
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

/// Bevy asset for [`rusty_spine::SkeletonData`], loaded asynchronously from [`Atlas`] and a
/// skeleton (either [`SkeletonJson`] or [`SkeletonBinary`]).
///
/// See [`SkeletonData::new_from_json`] or [`SkeletonData::new_from_binary`].
#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "7796a37b-37a4-49ea-bf4e-fb7344aa6015"]
pub struct SkeletonData {
    pub atlas_handle: Handle<Atlas>,
    pub kind: SkeletonDataKind,
    pub status: SkeletonDataStatus,
    pub premultiplied_alpha: bool,
}

#[derive(Debug)]
pub enum SkeletonDataKind {
    BinaryFile(Handle<SkeletonBinary>),
    JsonFile(Handle<SkeletonJson>),
}

#[derive(Debug)]
pub enum SkeletonDataStatus {
    Loaded(Arc<rusty_spine::SkeletonData>),
    Loading,
    Failed,
}

impl SkeletonData {
    /// Load a Spine skeleton from a JSON file.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_spine::prelude::*;
    ///
    /// // bevy system:
    /// fn load_skeleton_json(
    ///     mut skeletons: ResMut<Assets<SkeletonData>>,
    ///     asset_server: Res<AssetServer>,
    ///
    ///     mut commands: Commands,
    /// ) {
    ///     // load the skeleton (can be reused for multiple entities)
    ///     let skeleton = skeletons.add(SkeletonData::new_from_json(
    ///         asset_server.load("./skeleton.json"),
    ///         asset_server.load("./skeleton.atlas"),
    ///     ));
    ///
    ///     // to spawn the skeleton
    ///     commands.spawn(SpineBundle {
    ///         skeleton,
    ///         ..Default::default()
    ///     });
    /// }
    /// ```
    ///
    /// For more information on the loading process, see [`SpineBundle`](`crate::SpineBundle`).
    pub fn new_from_json(json: Handle<SkeletonJson>, atlas: Handle<Atlas>) -> Self {
        Self {
            atlas_handle: atlas,
            kind: SkeletonDataKind::JsonFile(json),
            status: SkeletonDataStatus::Loading,
            premultiplied_alpha: false,
        }
    }

    /// Load a Spine skeleton from a binary file.
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_spine::prelude::*;
    ///
    /// // bevy system:
    /// fn load_skeleton_binary(
    ///     mut skeletons: ResMut<Assets<SkeletonData>>,
    ///     asset_server: Res<AssetServer>,
    ///
    ///     mut commands: Commands,
    /// ) {
    ///     // load the skeleton (can be reused for multiple entities)
    ///     let skeleton = skeletons.add(SkeletonData::new_from_binary(
    ///         asset_server.load("./skeleton.skel"),
    ///         asset_server.load("./skeleton.atlas"),
    ///     ));
    ///
    ///     // to spawn the skeleton
    ///     commands.spawn(SpineBundle {
    ///         skeleton,
    ///         ..Default::default()
    ///     });
    /// }
    /// ```
    ///
    /// For more information on the loading process, see [`SpineBundle`](`crate::SpineBundle`).
    pub fn new_from_binary(binary: Handle<SkeletonBinary>, atlas: Handle<Atlas>) -> Self {
        Self {
            atlas_handle: atlas,
            kind: SkeletonDataKind::BinaryFile(binary),
            status: SkeletonDataStatus::Loading,
            premultiplied_alpha: false,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(&self.status, SkeletonDataStatus::Loaded(..))
    }

    pub fn skeleton_data(&self) -> Option<Arc<rusty_spine::SkeletonData>> {
        match &self.status {
            SkeletonDataStatus::Loaded(skeleton_data) => Some(skeleton_data.clone()),
            _ => None,
        }
    }
}
