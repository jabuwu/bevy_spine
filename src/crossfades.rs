use std::collections::HashMap;

use bevy::prelude::*;
use rusty_spine::AnimationStateData;

#[derive(Component, Default)]
pub struct Crossfades {
    mix_durations: HashMap<(String, String), f32>,
}

impl Crossfades {
    pub fn new() -> Self {
        Self {
            mix_durations: HashMap::new(),
        }
    }

    pub fn add(&mut self, from: &str, to: &str, mix_duration: f32) {
        self.mix_durations
            .insert((from.to_owned(), to.to_owned()), mix_duration);
    }

    pub(crate) fn apply(&self, animation_state_data: &mut AnimationStateData) {
        for ((from, to), mix_duration) in self.mix_durations.iter() {
            animation_state_data.set_mix_by_name(from, to, *mix_duration);
        }
    }
}
