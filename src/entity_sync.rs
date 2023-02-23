use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use bevy::prelude::*;

use crate::{Spine, SpineBone, SpineSystem};

#[cfg(doc)]
use crate::SpinePlugin;

pub trait SpineSynchronizer: Component + Clone + Eq + Debug + Hash {}
impl<T> SpineSynchronizer for T where T: Component + Clone + Eq + Debug + Hash {}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, SystemSet)]
pub enum SpineSynchronizerSystem<T: SpineSynchronizer> {
    SyncEntities,
    SyncBones,
    SyncEntitiesApplied,
    #[system_set(ignore_fields)]
    _Data(PhantomData<T>),
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy, SystemSet)]
pub enum SpineSynchronizerSet<T: SpineSynchronizer> {
    BeforeSync,
    DuringSync,
    AfterSync,
    #[system_set(ignore_fields)]
    _Data(PhantomData<T>),
}

/// A plugin for synchronizing [`SpineBone`] components with a rig.
///
/// This plugin is added automatically in [`SpinePlugin`] for [`SpineSync`] and does not need to be
/// added manually. However, custom synchronization steps can be added to allow for multiple syncs
/// in a single frame.
///
/// ```
/// # use bevy::prelude::*;
/// use bevy_spine::{prelude::*, SpineSynchronizerSet, SpineSynchronizerPlugin};
///
/// #[derive(Component, Debug, Hash, Clone, Copy, PartialEq, Eq)]
/// pub struct MySpineSync;
/// pub type MySpineSyncSet = SpineSynchronizerSet<MySpineSync>;
/// pub type MySpineSyncPlugin = SpineSynchronizerPlugin<MySpineSync, SpineSyncSet>; // add after SpineSync
///
/// # fn doc() {
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(SpinePlugin)
///         .add_plugin(MySpineSyncPlugin::default())
///         .add_system(spawn)
///         .add_system(during_sync.in_set(SpineSyncSet::DuringSync))
///         .add_system(during_my_sync.in_set(MySpineSyncSet::DuringSync))
///         // ...
///         .run();
/// }
/// # }
///
/// fn spawn(mut commands: Commands) {
///     // .. load spine ..
///     commands.spawn((
///         SpineBundle {
///             // ..
///             ..Default::default()
///         },
///         // synchronize in both steps
///         SpineSync,
///         MySpineSync,
///     ));
/// }
///
/// fn during_sync() {
///     // runs first
/// }
///
/// fn during_my_sync() {
///     // runs second
/// }
/// ```
pub struct SpineSynchronizerPlugin<T: SpineSynchronizer, After: SystemSet + Copy> {
    after: After,
    _marker: PhantomData<T>,
}

impl<T: SpineSynchronizer, S: SpineSynchronizer> Default
    for SpineSynchronizerPlugin<T, SpineSynchronizerSet<S>>
where
    SpineSynchronizerSet<S>: Copy,
{
    fn default() -> Self {
        Self {
            after: SpineSynchronizerSet::<S>::AfterSync,
            _marker: Default::default(),
        }
    }
}

impl<T: SpineSynchronizer> SpineSynchronizerPlugin<T, SpineSystem> {
    pub(crate) fn first() -> Self {
        Self {
            after: SpineSystem::Update,
            _marker: Default::default(),
        }
    }
}

impl<T: SpineSynchronizer, A: SystemSet + Copy> Plugin for SpineSynchronizerPlugin<T, A> {
    fn build(&self, app: &mut App) {
        app.add_system(
            spine_sync_entities::<T>
                .in_set(SpineSynchronizerSystem::<T>::SyncEntities)
                .after(self.after)
                .after(SpineSynchronizerSet::<T>::BeforeSync)
                .before(SpineSynchronizerSet::<T>::DuringSync),
        )
        .add_system(
            spine_sync_bones::<T>
                .in_set(SpineSynchronizerSystem::<T>::SyncBones)
                .after(SpineSynchronizerSystem::<T>::SyncEntities)
                .after(SpineSynchronizerSet::<T>::DuringSync),
        )
        .add_system(
            spine_sync_entities_applied::<T>
                .in_set(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
                .after(SpineSynchronizerSystem::<T>::SyncBones)
                .before(SpineSynchronizerSet::<T>::AfterSync)
                .before(SpineSystem::Render),
        );
    }
}

pub fn spine_sync_entities<S: SpineSynchronizer>(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    spine_query: Query<&Spine, With<S>>,
) {
    for (mut bone_transform, bone) in bone_query.iter_mut() {
        if let Ok(spine) = spine_query.get(bone.spine_entity) {
            if let Some(bone) = bone.handle.get(&spine.skeleton) {
                bone_transform.translation.x = bone.x();
                bone_transform.translation.y = bone.y();
                bone_transform.rotation =
                    Quat::from_axis_angle(Vec3::Z, bone.rotation().to_radians());
                bone_transform.scale.x = bone.scale_x();
                bone_transform.scale.y = bone.scale_y();
            }
        }
    }
}

pub fn spine_sync_bones<S: SpineSynchronizer>(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    mut spine_query: Query<&mut Spine, With<S>>,
) {
    for (bone_transform, bone) in bone_query.iter_mut() {
        if let Ok(mut spine) = spine_query.get_mut(bone.spine_entity) {
            if let Some(mut bone) = bone.handle.get_mut(&mut spine.skeleton) {
                bone.set_x(bone_transform.translation.x);
                bone.set_y(bone_transform.translation.y);
                let ang = bone_transform.rotation * Vec3::X;
                bone.set_rotation(ang.y.atan2(ang.x).to_degrees());
                bone.set_scale_x(bone_transform.scale.x);
                bone.set_scale_y(bone_transform.scale.y);
            }
        }
    }
    for mut spine in spine_query.iter_mut() {
        spine.0.skeleton.update_world_transform();
    }
}

pub fn spine_sync_entities_applied<S: SpineSynchronizer>(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    spine_query: Query<&Spine, With<S>>,
) {
    for (mut bone_transform, bone) in bone_query.iter_mut() {
        if let Ok(spine) = spine_query.get(bone.spine_entity) {
            if let Some(bone) = bone.handle.get(&spine.skeleton) {
                bone_transform.translation.x = bone.applied_x();
                bone_transform.translation.y = bone.applied_y();
                bone_transform.rotation =
                    Quat::from_axis_angle(Vec3::Z, bone.applied_rotation().to_radians());
                bone_transform.scale.x = bone.applied_scale_x();
                bone_transform.scale.y = bone.applied_scale_y();
            }
        }
    }
}

/// A [`Component`] which synchronizes child (bone) entities to to a [`Spine`] rig (see
/// [`SpineBone`]).
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_spine::{SpineLoader, SpineBundle, SpineSync};
/// # fn doc(mut commands: Commands) {
/// commands.spawn((
///     SpineBundle {
///         // ..
///         ..Default::default()
///     },
///     SpineSync
/// ));
/// # }
/// ```
///
/// If multiple synchronization steps are needed, additional sync components can be created (see
/// [`SpineSynchronizerPlugin`]).
#[derive(Component, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct SpineSync;

pub type SpineSyncSystem = SpineSynchronizerSystem<SpineSync>;
pub type SpineSyncSet = SpineSynchronizerSet<SpineSync>;
pub(crate) type SpineSyncPlugin = SpineSynchronizerPlugin<SpineSync, SpineSystem>;
