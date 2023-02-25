/// Enable feature `workaround_5732` in WASM builds to workaround Bevy issue 5732.
/// https://github.com/bevyengine/bevy/issues/5732
use std::collections::HashSet;
use std::mem::MaybeUninit;
use std::sync::{Mutex, Once};

use bevy::prelude::*;

static mut HANDLES: MaybeUninit<Mutex<HashSet<HandleUntyped>>> = MaybeUninit::uninit();
static ONCE: Once = Once::new();

pub(crate) fn store(handle: HandleUntyped) {
    let handles = unsafe {
        ONCE.call_once(|| {
            let singleton = Mutex::new(HashSet::new());
            HANDLES.write(singleton);
        });
        HANDLES.assume_init_ref()
    };
    let mut guard = handles.lock().expect("expected to lock mutex");
    guard.insert(handle);
}
