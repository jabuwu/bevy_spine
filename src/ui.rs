use bevy::{
    asset::RenderAssetUsages,
    camera::{ClearColorConfig, RenderTarget, visibility::RenderLayers},
    image::Image,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    ui::{ContentSize, widget::ViewportNode},
};

use crate::{
    Crossfades, SkeletonController, SkeletonData, SkeletonDataHandle, Spine, SpineBone,
    SpineLoader, SpineMesh, SpineReadyEvent, SpineRenderOwner, SpineSettings,
};

pub struct SpineUiPlugin;

#[derive(Resource, Default)]
struct SpineUiRenderLayerManager {
    owner_layers: HashMap<Entity, usize>,
    external_by_entity: HashMap<Entity, RenderLayers>,
    external_layer_use: HashMap<usize, usize>,
    pending_reassign: HashSet<Entity>,
}

impl SpineUiRenderLayerManager {
    fn allocate_for_owner(&mut self, owner: Entity) -> usize {
        if let Some(layer) = self.owner_layers.get(&owner) {
            return *layer;
        }

        let layer = self.find_free_layer(None);
        self.owner_layers.insert(owner, layer);
        layer
    }

    fn release_owner(&mut self, owner: Entity) {
        self.owner_layers.remove(&owner);
        self.pending_reassign.remove(&owner);
    }

    fn upsert_external_layers(&mut self, entity: Entity, layers: &RenderLayers) {
        if let Some(previous) = self.external_by_entity.insert(entity, layers.clone()) {
            for layer in previous.iter() {
                if let Some(count) = self.external_layer_use.get_mut(&layer) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.external_layer_use.remove(&layer);
                    }
                }
            }
        }

        for layer in layers.iter() {
            *self.external_layer_use.entry(layer).or_insert(0) += 1;
        }

        self.mark_conflicts(layers);
    }

    fn remove_external_entity(&mut self, entity: Entity) {
        if let Some(previous) = self.external_by_entity.remove(&entity) {
            for layer in previous.iter() {
                if let Some(count) = self.external_layer_use.get_mut(&layer) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.external_layer_use.remove(&layer);
                    }
                }
            }
        }
    }

    fn reassign_owner_layer(&mut self, owner: Entity) -> Option<(usize, usize)> {
        let current = self.owner_layers.get(&owner).copied()?;
        let next = self.find_free_layer(Some(owner));
        if current == next {
            return None;
        }

        self.owner_layers.insert(owner, next);
        Some((current, next))
    }

    fn take_pending_reassignments(&mut self) -> Vec<Entity> {
        self.pending_reassign.drain().collect()
    }

    fn mark_conflicts(&mut self, layers: &RenderLayers) {
        let used_layers: HashSet<usize> = layers.iter().collect();
        for (&owner, &owner_layer) in &self.owner_layers {
            if used_layers.contains(&owner_layer) {
                self.pending_reassign.insert(owner);
            }
        }
    }

    fn find_free_layer(&self, exempt_owner: Option<Entity>) -> usize {
        let mut layer = 1;
        loop {
            let used_by_external = self.external_layer_use.contains_key(&layer);
            let used_by_internal = self
                .owner_layers
                .iter()
                .any(|(owner, owner_layer)| Some(*owner) != exempt_owner && *owner_layer == layer);
            if !used_by_external && !used_by_internal {
                return layer;
            }
            layer += 1;
        }
    }
}

impl Plugin for SpineUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpineUiNode>()
            .register_type::<SpineUiSkeleton>()
            .register_type::<SpineUiFit>()
            .register_type::<SpineUiAnimation>()
            .register_type::<SpineUiProxy>()
            .register_type::<SpineUiDebugState>()
            .register_type::<SpineUiOwnedBy>()
            .init_resource::<SpineUiRenderLayerManager>()
            .add_message::<SpineUiReadyEvent>()
            .add_observer(on_external_render_layers_removed)
            .add_systems(
                Update,
                (
                    bootstrap_external_render_layers,
                    sync_changed_external_render_layers,
                    setup_spine_ui_nodes,
                    resolve_spine_ui_render_layer_conflicts,
                    update_spine_ui_content_size,
                    sync_spine_ui_proxies,
                    forward_spine_ui_ready_events,
                    sync_spine_ui_animation_changes,
                    cleanup_spine_ui_proxies,
                )
                    .chain(),
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

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component, Default)]
pub enum SpineUiFit {
    #[default]
    Contain,
    Cover,
    Fill,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Reflect)]
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

#[derive(Component, Clone, Debug, Default)]
struct SpineUiAnimationState {
    last_applied: Option<SpineUiAnimation>,
    last_non_animation: Option<SpineUiNonAnimationState>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SpineUiNonAnimationState {
    fit: SpineUiFit,
    auto_size: Option<Vec2>,
    reference_size: Option<Vec2>,
    offset: Vec2,
    scale: f32,
    flip_y: bool,
    tint: Color,
}

impl From<&SpineUiNode> for SpineUiNonAnimationState {
    fn from(node: &SpineUiNode) -> Self {
        Self {
            fit: node.fit,
            auto_size: node.auto_size,
            reference_size: node.reference_size,
            offset: node.offset,
            scale: node.scale,
            flip_y: node.flip_y,
            tint: node.tint,
        }
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub struct SpineUiReadyEvent {
    pub entity: Entity,
    pub proxy_entity: Entity,
}

#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component, Clone)]
struct SpineUiOwnedBy(Entity);

type ExternalRenderLayerEntityFilter = (Without<SpineUiOwnedBy>, Without<SpineRenderOwner>);

fn on_external_render_layers_removed(
    remove: On<Remove, RenderLayers>,
    mut manager: ResMut<SpineUiRenderLayerManager>,
) {
    manager.remove_external_entity(remove.entity);
}

fn bootstrap_external_render_layers(
    mut initialized: Local<bool>,
    mut manager: ResMut<SpineUiRenderLayerManager>,
    layers: Query<(Entity, &RenderLayers), ExternalRenderLayerEntityFilter>,
) {
    if *initialized {
        return;
    }

    for (entity, layers) in &layers {
        manager.upsert_external_layers(entity, layers);
    }

    *initialized = true;
}

fn sync_changed_external_render_layers(
    mut manager: ResMut<SpineUiRenderLayerManager>,
    changed_layers: Query<
        (Entity, &RenderLayers),
        (ExternalRenderLayerEntityFilter, Changed<RenderLayers>),
    >,
) {
    for (entity, layers) in &changed_layers {
        manager.upsert_external_layers(entity, layers);
    }
}

#[allow(clippy::type_complexity)]
fn setup_spine_ui_nodes(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layer_manager: ResMut<SpineUiRenderLayerManager>,
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
        let render_layer = layer_manager.allocate_for_owner(entity);
        let render_layers = RenderLayers::none().with(render_layer);

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
                    ..default()
                },
                RenderTarget::Image(image_handle.into()),
                render_layers.clone(),
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
                SpineRenderOwner,
                render_layers,
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
            SpineUiAnimationState::default(),
        ));

        if let Some(auto_size) = spine_ui.auto_size {
            entity_commands.insert(ContentSize::fixed_size(auto_size));
        }
    }
}

#[allow(clippy::type_complexity)]
fn resolve_spine_ui_render_layer_conflicts(
    mut manager: ResMut<SpineUiRenderLayerManager>,
    ui_nodes: Query<(), With<SpineUiNode>>,
    ui_proxies: Query<&SpineUiProxy>,
    mut layer_queries: ParamSet<(
        Query<(&SpineUiOwnedBy, &mut RenderLayers)>,
        Query<(&SpineMesh, &mut RenderLayers)>,
        Query<(&SpineBone, &mut RenderLayers)>,
    )>,
) {
    for owner in manager.take_pending_reassignments() {
        if !ui_nodes.contains(owner) {
            manager.release_owner(owner);
            continue;
        }

        let Some((previous_layer, new_layer)) = manager.reassign_owner_layer(owner) else {
            continue;
        };
        let render_layers = RenderLayers::none().with(new_layer);

        {
            let mut owned_layers = layer_queries.p0();
            for (owned_by, mut layers) in &mut owned_layers {
                if owned_by.0 == owner {
                    *layers = render_layers.clone();
                }
            }
        }

        let mut updated_renderables = 0usize;
        if let Ok(proxy) = ui_proxies.get(owner) {
            {
                let mut mesh_layers = layer_queries.p1();
                for (spine_mesh, mut layers) in &mut mesh_layers {
                    if spine_mesh.spine_entity == proxy.proxy_entity {
                        *layers = render_layers.clone();
                        updated_renderables += 1;
                    }
                }
            }

            {
                let mut bone_layers = layer_queries.p2();
                for (spine_bone, mut layers) in &mut bone_layers {
                    if spine_bone.spine_entity == proxy.proxy_entity {
                        *layers = render_layers.clone();
                        updated_renderables += 1;
                    }
                }
            }
        }

        bevy::log::warn!(
            "Reassigned bevy_spine UI owner {owner:?} from RenderLayer {previous_layer} to {new_layer} to avoid collision with external usage (updated {updated_renderables} proxy renderables)."
        );
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
    mut animation_state_query: Query<&mut SpineUiAnimationState>,
    mut spine_query: Query<&mut Spine>,
) {
    for event in spine_ready_events.read() {
        let Ok(owner) = owner_query.get(event.entity) else {
            continue;
        };

        if let Ok(spine_ui) = node_query.get(owner.0)
            && let Ok(mut spine) = spine_query.get_mut(event.entity)
        {
            let Spine(SkeletonController {
                animation_state,
                skeleton,
                ..
            }) = spine.as_mut();
            let [r, g, b, a] = spine_ui.tint.to_linear().to_f32_array();
            *skeleton.color_mut() = rusty_spine::Color::new_rgba(r, g, b, a);

            if let Some(animation) = spine_ui.animation.as_ref() {
                let _ = animation_state.set_animation_by_name(
                    0,
                    animation.name.as_str(),
                    animation.repeat,
                );
            } else {
                animation_state.clear_track(0);
            }

            if let Ok(mut applied) = animation_state_query.get_mut(owner.0) {
                applied.last_applied = spine_ui.animation.clone();
                applied.last_non_animation = Some(SpineUiNonAnimationState::from(spine_ui));
            }
        }

        spine_ui_ready_events.write(SpineUiReadyEvent {
            entity: owner.0,
            proxy_entity: event.entity,
        });
    }
}

fn sync_spine_ui_animation_changes(
    mut nodes: Query<
        (&SpineUiNode, &SpineUiProxy, &mut SpineUiAnimationState),
        Changed<SpineUiNode>,
    >,
    mut spine_query: Query<&mut Spine>,
) {
    for (spine_ui, proxy, mut animation_state) in &mut nodes {
        let non_animation_state = SpineUiNonAnimationState::from(spine_ui);
        let animation_unchanged = animation_state.last_applied == spine_ui.animation;
        let non_animation_changed = animation_state.last_non_animation != Some(non_animation_state);

        if animation_unchanged && non_animation_changed {
            animation_state.last_non_animation = Some(non_animation_state);
            continue;
        }

        let Ok(mut spine) = spine_query.get_mut(proxy.proxy_entity) else {
            continue;
        };

        let Spine(SkeletonController {
            animation_state: spine_animation_state,
            ..
        }) = spine.as_mut();

        if let Some(animation) = spine_ui.animation.as_ref() {
            let _ = spine_animation_state.set_animation_by_name(
                0,
                animation.name.as_str(),
                animation.repeat,
            );
        } else {
            spine_animation_state.clear_track(0);
        }

        animation_state.last_applied = spine_ui.animation.clone();
        animation_state.last_non_animation = Some(non_animation_state);
    }
}

#[allow(clippy::type_complexity)]
fn cleanup_spine_ui_proxies(
    mut commands: Commands,
    mut layer_manager: ResMut<SpineUiRenderLayerManager>,
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
                With<SpineUiAnimationState>,
            )>,
        ),
    >,
) {
    for (entity, owner) in &owners {
        if ui_nodes.contains(owner.0) {
            continue;
        }

        layer_manager.release_owner(owner.0);

        commands.entity(entity).despawn();

        if let Ok(mut owner_commands) = commands.get_entity(owner.0) {
            owner_commands.remove::<(SpineUiProxy, ViewportNode)>();
        }
    }

    for entity in &stale_ui_nodes {
        commands.entity(entity).remove::<(
            SpineUiProxy,
            ViewportNode,
            SpineUiDebugState,
            SpineUiAnimationState,
        )>();
    }
}
