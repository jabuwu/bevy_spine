use bevy::{
    asset::RenderAssetUsages,
    camera::{ClearColorConfig, RenderTarget},
    image::Image,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    ui::{ContentSize, widget::ViewportNode},
};

use crate::{
    Crossfades, SkeletonController, SkeletonData, SkeletonDataHandle, Spine, SpineLoader,
    SpineReadyEvent, SpineSettings,
};

pub struct SpineUiPlugin;

impl Plugin for SpineUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpineUiNode>()
            .register_type::<SpineUiSkeleton>()
            .register_type::<SpineUiFit>()
            .register_type::<SpineUiAnimation>()
            .register_type::<SpineUiProxy>()
            .register_type::<SpineUiDebugState>()
            .register_type::<SpineUiOwnedBy>()
            .add_message::<SpineUiReadyEvent>()
            .add_systems(
                Update,
                (
                    setup_spine_ui_nodes,
                    update_spine_ui_content_size,
                    sync_spine_ui_proxies,
                    forward_spine_ui_ready_events,
                    cleanup_spine_ui_proxies,
                ),
            );
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[require(Node, Crossfades, SpineSettings)]
#[reflect(Component, Default)]
pub struct SpineUiNode {
    pub fit: SpineUiFit,
    pub auto_size: Option<Vec2>,
    pub reference_size: Option<Vec2>,
    pub offset: Vec2,
    pub scale: f32,
    pub flip_y: bool,
    pub tint: Color,
    pub animation: Option<SpineUiAnimation>,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Default, Clone)]
pub struct SpineUiSkeleton(pub Handle<SkeletonData>);

impl Default for SpineUiSkeleton {
    fn default() -> Self {
        Self(default())
    }
}

impl Default for SpineUiNode {
    fn default() -> Self {
        Self {
            fit: SpineUiFit::Contain,
            auto_size: Some(Vec2::new(300.0, 420.0)),
            reference_size: None,
            offset: Vec2::ZERO,
            scale: 1.0,
            flip_y: false,
            tint: Color::WHITE,
            animation: None,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub enum SpineUiFit {
    #[default]
    Contain,
    Cover,
    Fill,
    None,
}

#[derive(Clone, Debug, Reflect)]
pub struct SpineUiAnimation {
    pub name: String,
    pub repeat: bool,
}

impl SpineUiAnimation {
    pub fn looping(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            repeat: true,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct SpineUiProxy {
    pub proxy_entity: Entity,
    pub camera_entity: Entity,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component, Default)]
struct SpineUiDebugState;

#[derive(Message, Clone, Copy, Debug)]
pub struct SpineUiReadyEvent {
    pub entity: Entity,
    pub proxy_entity: Entity,
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component, Clone)]
struct SpineUiOwnedBy(Entity);

#[allow(clippy::type_complexity)]
fn setup_spine_ui_nodes(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    nodes: Query<
        (
            Entity,
            &SpineUiSkeleton,
            &Crossfades,
            &SpineSettings,
            &SpineUiNode,
        ),
        (Added<SpineUiNode>, Without<SpineUiProxy>),
    >,
) {
    for (entity, skeleton, crossfades, settings, spine_ui) in &nodes {
        let mut image = Image::new_uninit(
            Extent3d {
                width: 1,
                height: 1,
                ..default()
            },
            TextureDimension::D2,
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::all(),
        );
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;
        let image_handle = images.add(image);

        let camera_entity = commands
            .spawn((
                Camera2d,
                Camera {
                    order: -1,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    target: RenderTarget::Image(image_handle.into()),
                    ..default()
                },
                SpineUiOwnedBy(entity),
            ))
            .id();

        let mut proxy_settings = *settings;
        proxy_settings.mesh_type = crate::SpineMeshType::Mesh2D;

        let proxy_entity = commands
            .spawn((
                SkeletonDataHandle(skeleton.0.clone()),
                crossfades.clone(),
                proxy_settings,
                SpineLoader::without_children(),
                SpineUiOwnedBy(entity),
            ))
            .id();

        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((
            ViewportNode::new(camera_entity),
            SpineUiProxy {
                proxy_entity,
                camera_entity,
            },
        ));

        if let Some(auto_size) = spine_ui.auto_size {
            entity_commands.insert(ContentSize::fixed_size(auto_size));
        }
    }
}

fn update_spine_ui_content_size(
    mut commands: Commands,
    nodes: Query<(Entity, &SpineUiNode), Changed<SpineUiNode>>,
) {
    for (entity, spine_ui) in &nodes {
        if let Some(auto_size) = spine_ui.auto_size {
            commands
                .entity(entity)
                .insert(ContentSize::fixed_size(auto_size));
        } else {
            commands.entity(entity).remove::<ContentSize>();
        }
    }
}

fn sync_spine_ui_proxies(
    mut nodes: Query<(
        &ComputedNode,
        &InheritedVisibility,
        &SpineUiNode,
        &SpineUiProxy,
    )>,
    mut proxy_query: Query<(&mut Transform, &mut Visibility, &mut Spine)>,
    mut camera_query: Query<&mut Camera>,
) {
    for (computed_node, inherited_visibility, spine_ui, proxy) in &mut nodes {
        if let Ok(mut camera) = camera_query.get_mut(proxy.camera_entity) {
            camera.is_active = inherited_visibility.get();
        }

        let Ok((mut proxy_transform, mut proxy_visibility, mut spine)) =
            proxy_query.get_mut(proxy.proxy_entity)
        else {
            continue;
        };

        let Spine(SkeletonController { skeleton, .. }) = spine.as_mut();
        let [r, g, b, a] = spine_ui.tint.to_linear().to_f32_array();
        *skeleton.color_mut() = rusty_spine::Color::new_rgba(r, g, b, a);

        let data = skeleton.data();
        let setup_min = Vec2::new(data.x(), data.y());
        let setup_size = Vec2::new(data.width(), data.height()).max(Vec2::ONE);
        let setup_center = setup_min + setup_size * 0.5;

        *proxy_visibility = if inherited_visibility.get() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        let available_size = computed_node.size().max(Vec2::ONE);
        let reference_size = spine_ui.reference_size.unwrap_or(setup_size).max(Vec2::ONE);
        let fit_scale = available_size / reference_size;

        let mut applied_scale = match spine_ui.fit {
            SpineUiFit::Contain => {
                let s = fit_scale.x.min(fit_scale.y) * spine_ui.scale;
                Vec2::new(s, s)
            }
            SpineUiFit::Cover => {
                let s = fit_scale.x.max(fit_scale.y) * spine_ui.scale;
                Vec2::new(s, s)
            }
            SpineUiFit::Fill => {
                Vec2::new(fit_scale.x * spine_ui.scale, fit_scale.y * spine_ui.scale)
            }
            SpineUiFit::None => Vec2::splat(spine_ui.scale),
        };
        if spine_ui.flip_y {
            applied_scale.y *= -1.0;
        }

        let centered_translation = Vec2::new(
            -setup_center.x * applied_scale.x,
            -setup_center.y * applied_scale.y,
        ) + spine_ui.offset;

        *proxy_transform = Transform::from_translation(centered_translation.extend(0.0))
            .with_scale(applied_scale.extend(1.0));
    }
}

fn forward_spine_ui_ready_events(
    mut spine_ready_events: MessageReader<SpineReadyEvent>,
    mut spine_ui_ready_events: MessageWriter<SpineUiReadyEvent>,
    owner_query: Query<&SpineUiOwnedBy>,
    node_query: Query<&SpineUiNode>,
    mut spine_query: Query<&mut Spine>,
) {
    for event in spine_ready_events.read() {
        let Ok(owner) = owner_query.get(event.entity) else {
            continue;
        };

        if let Ok(spine_ui) = node_query.get(owner.0) {
            if let Some(animation) = spine_ui.animation.as_ref() {
                if let Ok(mut spine) = spine_query.get_mut(event.entity) {
                    let Spine(SkeletonController {
                        animation_state,
                        skeleton,
                        ..
                    }) = spine.as_mut();
                    let [r, g, b, a] = spine_ui.tint.to_linear().to_f32_array();
                    *skeleton.color_mut() = rusty_spine::Color::new_rgba(r, g, b, a);
                    let _ = animation_state.set_animation_by_name(
                        0,
                        animation.name.as_str(),
                        animation.repeat,
                    );
                }
            }
        }

        spine_ui_ready_events.write(SpineUiReadyEvent {
            entity: owner.0,
            proxy_entity: event.entity,
        });
    }
}

#[allow(clippy::type_complexity)]
fn cleanup_spine_ui_proxies(
    mut commands: Commands,
    owners: Query<(Entity, &SpineUiOwnedBy)>,
    ui_nodes: Query<(), With<SpineUiNode>>,
    stale_ui_nodes: Query<
        Entity,
        (
            Without<SpineUiNode>,
            Or<(
                With<SpineUiProxy>,
                With<ViewportNode>,
                With<SpineUiDebugState>,
            )>,
        ),
    >,
) {
    for (entity, owner) in &owners {
        if ui_nodes.contains(owner.0) {
            continue;
        }

        commands.entity(entity).despawn();

        if let Ok(mut owner_commands) = commands.get_entity(owner.0) {
            owner_commands.remove::<(SpineUiProxy, ViewportNode)>();
        }
    }

    for entity in &stale_ui_nodes {
        commands
            .entity(entity)
            .remove::<(SpineUiProxy, ViewportNode, SpineUiDebugState)>();
    }
}
