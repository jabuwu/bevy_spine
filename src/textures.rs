use std::sync::{Arc, Mutex};

use bevy::prelude::*;

#[derive(Debug)]
pub struct SpineTexture(pub String);

pub(crate) struct SpineTextures {
    handles: Arc<Mutex<Vec<(String, Handle<Image>)>>>,
    remember: Arc<Mutex<Vec<String>>>,
    forget: Arc<Mutex<Vec<String>>>,
}

impl SpineTextures {
    pub(crate) fn init() -> Self {
        let handles: Arc<Mutex<Vec<(String, Handle<Image>)>>> = Arc::new(Mutex::new(Vec::new()));
        let remember: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let forget: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let remember2 = remember.clone();
        rusty_spine::extension::set_create_texture_cb(move |page, path| {
            remember2.lock().unwrap().push(path.to_owned());
            page.renderer_object().set(SpineTexture(path.to_owned()));
        });

        let forget2 = forget.clone();
        rusty_spine::extension::set_dispose_texture_cb(move |page| unsafe {
            forget2.lock().unwrap().push(
                page.renderer_object()
                    .get_unchecked::<SpineTexture>()
                    .0
                    .clone(),
            );
            page.renderer_object().dispose::<SpineTexture>();
        });

        Self {
            handles,
            remember,
            forget,
        }
    }

    pub fn update(&self, asset_server: &AssetServer) {
        let mut handles = self.handles.lock().unwrap();
        let mut remember = self.remember.lock().unwrap();
        let mut forget = self.forget.lock().unwrap();
        while let Some(image) = remember.pop() {
            handles.push((image.clone(), asset_server.load(&image)));
        }
        while let Some(image) = forget.pop() {
            if let Some(index) = handles.iter().position(|i| i.0 == image) {
                handles.remove(index);
            }
        }
    }
}
