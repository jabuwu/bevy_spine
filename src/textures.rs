use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    render::{
        render_resource::{FilterMode, SamplerDescriptor},
        texture::ImageSampler,
    },
};

#[derive(Debug)]
pub(crate) struct SpineTexture(pub String);

pub(crate) struct SpineTextures {
    data: Arc<Mutex<SpineTexturesData>>,
}

#[derive(Debug, Clone)]
pub struct SpineTextureCreateEvent {
    pub path: String,
    pub handle: Handle<Image>,
}

#[derive(Debug, Clone)]
pub struct SpineTextureDisposeEvent {
    pub path: String,
    pub handle: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct SpineTexturesData {
    handles: Vec<(String, Handle<Image>)>,
    initialize: Vec<Handle<Image>>,
    remember: Vec<String>,
    forget: Vec<String>,
}

impl SpineTextures {
    pub(crate) fn init() -> Self {
        let data = Arc::new(Mutex::new(SpineTexturesData::default()));

        let data2 = data.clone();
        rusty_spine::extension::set_create_texture_cb(move |page, path| {
            data2.lock().unwrap().remember.push(path.to_owned());
            page.renderer_object().set(SpineTexture(path.to_owned()));
        });

        let data3 = data.clone();
        rusty_spine::extension::set_dispose_texture_cb(move |page| unsafe {
            data3.lock().unwrap().forget.push(
                page.renderer_object()
                    .get_unchecked::<SpineTexture>()
                    .0
                    .clone(),
            );
            page.renderer_object().dispose::<SpineTexture>();
        });

        Self { data }
    }

    pub fn update(
        &self,
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        create_events: &mut EventWriter<SpineTextureCreateEvent>,
        dispose_events: &mut EventWriter<SpineTextureDisposeEvent>,
    ) {
        let mut data = self.data.lock().unwrap();
        while let Some(image) = data.remember.pop() {
            let handle = asset_server.load(&image);
            data.handles.push((image.clone(), handle.clone()));
            data.initialize.push(handle.clone());
            create_events.send(SpineTextureCreateEvent {
                path: image,
                handle,
            });
        }
        while let Some(image) = data.forget.pop() {
            if let Some(index) = data.handles.iter().position(|i| i.0 == image) {
                let initialize_position = data
                    .initialize
                    .iter()
                    .position(|h| *h == data.handles[index].1);
                if let Some(initialize_position) = initialize_position {
                    data.initialize.remove(initialize_position);
                }
                dispose_events.send(SpineTextureDisposeEvent {
                    path: image,
                    handle: data.handles[index].1.clone(),
                });
                data.handles.remove(index);
            }
        }
        let mut remove_initialize = vec![];
        for (i, handle) in data.initialize.iter().enumerate() {
            if let Some(image) = images.get_mut(handle) {
                image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    mag_filter: FilterMode::Nearest,
                    min_filter: FilterMode::Nearest,
                    ..Default::default()
                });
                remove_initialize.push(i);
            }
        }
        for remove in remove_initialize.iter().rev() {
            data.initialize.remove(*remove);
        }
    }
}
