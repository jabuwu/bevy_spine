use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use bevy::{
    ecs::{
        schedule::SystemConfig,
        system::{AlreadyWasSystem, BoxedSystem, FunctionSystem, IsFunctionSystem, SystemParam},
    },
    prelude::*,
};

use crate::{Spine, SpineBone, SpineSystem};

pub trait SpineSynchronizer: Component + Clone + Eq + Debug + Hash {}
impl<T> SpineSynchronizer for T where T: Component + Clone + Eq + Debug + Hash {}

#[derive(Hash, Debug, PartialEq, Eq, Clone, SystemSet)]
pub enum SpineSynchronizerSystem<T: SpineSynchronizer> {
    SyncEntities,
    SyncBones,
    SyncEntitiesApplied,
    #[system_set(ignore_fields)]
    _Data(T),
}

pub struct SpineSynchronizerPlugin<T: SpineSynchronizer> {
    _marker: PhantomData<T>,
}

impl<T: SpineSynchronizer> Default for SpineSynchronizerPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<T: SpineSynchronizer> Plugin for SpineSynchronizerPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system(
            spine_sync_entities::<T>
                .in_set(SpineSynchronizerSystem::<T>::SyncEntities)
                .after(SpineSystem::Update),
        )
        .add_system(
            spine_sync_bones::<T>
                .in_set(SpineSynchronizerSystem::<T>::SyncBones)
                .after(SpineSynchronizerSystem::<T>::SyncEntities),
        )
        .add_system(
            spine_sync_entities_applied::<T>
                .in_set(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
                .after(SpineSynchronizerSystem::<T>::SyncBones)
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

pub trait IntoSpineSystem<In, Out, Params>: bevy::prelude::IntoSystem<In, Out, Params> {
    type System: System<In = In, Out = Out>;
}
impl<In, Out, Sys: System<In = In, Out = Out>> IntoSpineSystem<In, Out, AlreadyWasSystem> for Sys {
    type System = Sys;
}
impl<In, Out, Param, Marker, F> IntoSpineSystem<In, Out, (IsFunctionSystem, Param, Marker)> for F
where
    In: 'static,
    Out: 'static,
    Param: SystemParam + 'static,
    Marker: 'static,
    F: SystemParamFunction<In, Out, Param, Marker> + Send + Sync + 'static,
{
    type System = FunctionSystem<In, Out, Param, Marker, F>;
}

pub trait SpineSystemFunctions<Params> {
    fn before_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig;
    fn during_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig;
    fn after_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig;
}

impl SpineSystemFunctions<()> for SystemConfig {
    fn before_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSystem::Update)
            .before(SpineSynchronizerSystem::<T>::SyncEntities)
    }
    fn during_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSynchronizerSystem::<T>::SyncEntities)
            .before(SpineSynchronizerSystem::<T>::SyncBones)
    }
    fn after_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
            .before(SpineSystem::Render)
    }
}

impl<S, Params> SpineSystemFunctions<Params> for S
where
    S: IntoSpineSystem<(), (), Params>,
{
    fn before_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSystem::Update)
            .before(SpineSynchronizerSystem::<T>::SyncEntities)
    }
    fn during_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSynchronizerSystem::<T>::SyncEntities)
            .before(SpineSynchronizerSystem::<T>::SyncBones)
    }
    fn after_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
            .before(SpineSystem::Render)
    }
}

impl SpineSystemFunctions<()> for BoxedSystem<(), ()> {
    fn before_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSystem::Update)
            .before(SpineSynchronizerSystem::<T>::SyncEntities)
    }
    fn during_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSynchronizerSystem::<T>::SyncEntities)
            .before(SpineSynchronizerSystem::<T>::SyncBones)
    }
    fn after_spine_sync<T: SpineSynchronizer>(self) -> SystemConfig {
        self.after(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
            .before(SpineSystem::Render)
    }
}

#[derive(Component, Debug, Hash, Clone, PartialEq, Eq)]
pub struct SpineSync;

pub type SpineSyncSystem = SpineSynchronizerSystem<SpineSync>;
pub type SpineSyncPlugin = SpineSynchronizerPlugin<SpineSync>;
