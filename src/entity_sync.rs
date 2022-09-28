use bevy::prelude::*;

use crate::{Spine, SpineBone};

pub fn spine_sync_entities(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    spine_query: Query<&Spine>,
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

pub fn spine_sync_bones(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    mut spine_query: Query<&mut Spine>,
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

pub fn spine_sync_entities_applied(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    spine_query: Query<&Spine>,
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
