use std::marker::PhantomData;

use bevy::{
    ecs::{schedule::ParallelSystemDescriptor, system::BoxedSystem},
    prelude::*,
};

use crate::{Spine, SpineBone, SpineSystem};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum SpineSynchronizerSystem<T: Component> {
    SyncEntities,
    SyncBones,
    SyncEntitiesApplied,
    #[system_label(ignore_fields)]
    _Data(T),
}

pub struct SpineSynchronizerPlugin<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> Default for SpineSynchronizerPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<T: Component> Plugin for SpineSynchronizerPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_system(
            spine_sync_entities::<SpineSync>
                .label(SpineSynchronizerSystem::<T>::SyncEntities)
                .after(SpineSystem::Update),
        )
        .add_system(
            spine_sync_bones::<SpineSync>
                .label(SpineSynchronizerSystem::<T>::SyncBones)
                .after(SpineSynchronizerSystem::<T>::SyncEntities),
        )
        .add_system(
            spine_sync_entities_applied::<SpineSync>
                .label(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
                .after(SpineSynchronizerSystem::<T>::SyncBones)
                .before(SpineSystem::Render),
        );
    }
}

pub fn spine_sync_entities<S: Component>(
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

pub fn spine_sync_bones<S: Component>(
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

pub fn spine_sync_entities_applied<S: Component>(
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

pub trait IntoSpineSystem<In, Out, Params>: bevy::prelude::IntoSystem<In, Out, Params> {}

pub trait SpineSystemFunctions<Params> {
    fn before_spine_sync<T: Component>(self) -> ParallelSystemDescriptor;
    fn during_spine_sync<T: Component>(self) -> ParallelSystemDescriptor;
    fn after_spine_sync<T: Component>(self) -> ParallelSystemDescriptor;
}

impl SpineSystemFunctions<()> for ParallelSystemDescriptor {
    fn before_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSystem::Update)
            .before(SpineSynchronizerSystem::<T>::SyncEntities)
    }
    fn during_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSynchronizerSystem::<T>::SyncEntities)
            .before(SpineSynchronizerSystem::<T>::SyncBones)
    }
    fn after_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
            .before(SpineSystem::Render)
    }
}

impl<S, Params> SpineSystemFunctions<Params> for S
where
    S: IntoSpineSystem<(), (), Params>,
{
    fn before_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSystem::Update)
            .before(SpineSynchronizerSystem::<T>::SyncEntities)
    }
    fn during_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSynchronizerSystem::<T>::SyncEntities)
            .before(SpineSynchronizerSystem::<T>::SyncBones)
    }
    fn after_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
            .before(SpineSystem::Render)
    }
}

impl SpineSystemFunctions<()> for BoxedSystem<(), ()> {
    fn before_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSystem::Update)
            .before(SpineSynchronizerSystem::<T>::SyncEntities)
    }
    fn during_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSynchronizerSystem::<T>::SyncEntities)
            .before(SpineSynchronizerSystem::<T>::SyncBones)
    }
    fn after_spine_sync<T: Component>(self) -> ParallelSystemDescriptor {
        self.after(SpineSynchronizerSystem::<T>::SyncEntitiesApplied)
            .before(SpineSystem::Render)
    }
}

#[derive(Component)]
pub struct SpineSync;

pub type SpineSyncSystem = SpineSynchronizerSystem<SpineSync>;
pub type SpineSyncPlugin = SpineSynchronizerPlugin<SpineSync>;
