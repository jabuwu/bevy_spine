//! Events related to textures loaded by Spine.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::Atlas;

#[derive(Debug)]
pub(crate) struct SpineTexture(pub String);

#[derive(Debug)]
struct SpineTextureInternal {
    pub path: String,
    pub atlas_address: usize,
}

#[derive(Resource)]
pub(crate) struct SpineTextures {
    data: Arc<Mutex<SpineTexturesData>>,
}

/// An [`Event`] fired for each texture loaded by Spine.
#[derive(Debug, Clone)]
pub struct SpineTextureCreateEvent {
    pub path: String,
    pub handle: Handle<Image>,
    pub atlas: Handle<Atlas>,
}

/// An [`Event`] fired for each texture disposed, after [`SpineTextureCreateEvent`].
#[derive(Debug, Clone)]
pub struct SpineTextureDisposeEvent {
    pub path: String,
    pub handle: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct SpineTexturesData {
    handles: Vec<(String, Handle<Image>)>,
    remember: Vec<SpineTextureInternal>,
    forget: Vec<SpineTextureInternal>,
}

impl SpineTextures {
    pub(crate) fn init() -> Self {
        let data = Arc::new(Mutex::new(SpineTexturesData::default()));

        let data2 = data.clone();
        rusty_spine::extension::set_create_texture_cb(move |page, path| {
            data2.lock().unwrap().remember.push(SpineTextureInternal {
                path: path.to_owned(),
                atlas_address: page.atlas().c_ptr() as usize,
            });
            page.renderer_object().set(SpineTexture(path.to_owned()));
        });

        let data3 = data.clone();
        rusty_spine::extension::set_dispose_texture_cb(move |page| unsafe {
            data3.lock().unwrap().forget.push(SpineTextureInternal {
                path: page
                    .renderer_object()
                    .get_unchecked::<SpineTexture>()
                    .0
                    .clone(),
                atlas_address: page.atlas().c_ptr() as usize,
            });
            page.renderer_object().dispose::<SpineTexture>();
        });

        Self { data }
    }

    pub fn update(
        &self,
        asset_server: &AssetServer,
        atlases: &Assets<Atlas>,
        create_events: &mut EventWriter<SpineTextureCreateEvent>,
        dispose_events: &mut EventWriter<SpineTextureDisposeEvent>,
    ) {
        let mut data = self.data.lock().unwrap();
        while let Some(texture) = data.remember.pop() {
            let handle = asset_server.load(&texture.path);
            // if none, the atlas was already deleted before getting here
            if let Some(atlas) = find_matching_atlas(atlases, texture.atlas_address) {
                data.handles.push((texture.path.clone(), handle.clone()));
                create_events.send(SpineTextureCreateEvent {
                    path: texture.path,
                    atlas,
                    handle,
                });
            }
        }
        while let Some(texture) = data.forget.pop() {
            if let Some(index) = data.handles.iter().position(|i| i.0 == texture.path) {
                dispose_events.send(SpineTextureDisposeEvent {
                    path: texture.path,
                    handle: data.handles[index].1.clone(),
                });
                data.handles.remove(index);
            }
        }
    }
}

fn find_matching_atlas(atlases: &Assets<Atlas>, atlas_address: usize) -> Option<Handle<Atlas>> {
    for (atlas_handle, atlas) in atlases.iter() {
        if atlas.atlas.c_ptr() as usize == atlas_address {
            return Some(atlases.get_handle(atlas_handle));
        }
    }
    None
}
