use std::collections::HashMap;

use bevy::prelude::*;
use rusty_spine::AnimationStateData;

/// Crossfade data to apply to [`rusty_spine::AnimationStateData`]. Allows automated crossfading
/// between animations.
///
/// Apply to a [`SpineBundle`](`crate::SpineBundle`) upon creation:
///
/// ```

/// # use bevy::prelude::*;
/// # use bevy_spine::prelude::*;
/// # fn doc(mut commands: Commands) {
/// let mut crossfades = Crossfades::new();
///
/// // Blend between walk -> run for 0.2 secs
/// crossfades.add("walk", "run", 0.2);
///
/// // Apply in the other direction too
/// crossfades.add("run", "walk", 0.2);
///
/// commands.spawn(SpineBundle {
///     // ...
///     crossfades,
///     ..Default::default()
/// });
/// # }
/// ```

#[derive(Component, Default, Clone)]
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
