use crate::{Crossfades, SkeletonData, SpineLoader, SpineSettings};
use bevy::prelude::*;

/// Attach this component to an entity to load and spawn a Spine skeleton.
///
/// This component uses Bevy required components to automatically add
/// [`SpineLoader`], [`SpineSettings`], [`Crossfades`], [`Transform`], and
/// [`Visibility`] when they are not already present.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_spine::prelude::*;
/// # fn doc(mut commands: Commands, skeleton: Handle<SkeletonData>) {
/// commands.spawn((
///     SkeletonDataHandle(skeleton),
///     Transform::from_xyz(0.0, -200.0, 0.0),
/// ));
/// # }
/// ```
#[derive(Default, Component, Clone, Reflect)]
#[require(SpineLoader, SpineSettings, Crossfades, Transform, Visibility)]
#[reflect(Component, Default, Clone)]
pub struct SkeletonDataHandle(pub Handle<SkeletonData>);

impl From<Handle<SkeletonData>> for SkeletonDataHandle {
    fn from(handle: Handle<SkeletonData>) -> Self {
        Self(handle)
    }
}
