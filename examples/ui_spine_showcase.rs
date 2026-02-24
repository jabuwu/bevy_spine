use bevy::{prelude::*, ui_render::UiDebugOptions};
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpinePlugin, SpineUiAnimation, SpineUiFit,
    SpineUiNode, SpineUiProxy, SpineUiSkeleton,
};

fn main() {
    App::new()
        .insert_resource(ShowcaseState::default())
        .add_plugins((DefaultPlugins, SpinePlugin))
        .add_systems(Startup, (setup, enable_ui_debug_overlay))
        .add_systems(
            Update,
            (
                showcase_input,
                apply_showcase_state,
                toggle_ui_debug_overlay,
            ),
        )
        .run();
}

#[derive(Resource)]
struct ShowcaseState {
    fit: SpineUiFit,
    auto_size_enabled: bool,
    tint_index: usize,
    animation_index: usize,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            fit: SpineUiFit::Contain,
            auto_size_enabled: true,
            tint_index: 0,
            animation_index: 0,
        }
    }
}

#[derive(Component)]
struct InteractiveSpine;

#[derive(Component)]
struct AutoSizeSpine(Vec2);

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
) {
    commands.spawn(Camera2d);

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.08)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(
                    "Spine UI Showcase\n[1-4] fit: contain/cover/fill/none  [A] auto-size  [T] tint  [Space] animation  [F1] UI borders",
                ),
                TextColor(Color::srgb(0.9, 0.92, 0.97)),
            ));

            root.spawn(Node {
                width: percent(100),
                height: percent(100),
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::fr(1.0), GridTrack::fr(1.0)],
                grid_template_rows: vec![GridTrack::fr(1.0), GridTrack::fr(1.0)],
                row_gap: px(8),
                column_gap: px(8),
                ..default()
            })
            .with_children(|grid| {
                spawn_panel(
                    grid,
                    "Interactive (explicit size)",
                    skeleton_handle.clone(),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        min_height: px(220),
                        ..default()
                    },
                    SpineUiNode {
                        animation: Some(SpineUiAnimation::looping("walk")),
                        auto_size: None,
                        ..default()
                    },
                    Some(InteractiveSpine),
                    None,
                );

                spawn_panel(
                    grid,
                    "Auto-size node",
                    skeleton_handle.clone(),
                    Node::default(),
                    SpineUiNode {
                        animation: Some(SpineUiAnimation::looping("portal")),
                        auto_size: Some(Vec2::new(260.0, 380.0)),
                        ..default()
                    },
                    Some(InteractiveSpine),
                    Some(AutoSizeSpine(Vec2::new(260.0, 380.0))),
                );

                spawn_panel(
                    grid,
                    "Tint sample",
                    skeleton_handle.clone(),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        min_height: px(220),
                        ..default()
                    },
                    SpineUiNode {
                        animation: Some(SpineUiAnimation::looping("walk")),
                        tint: Color::srgb(1.0, 0.85, 0.75),
                        ..default()
                    },
                    Some(InteractiveSpine),
                    None,
                );

                grid.spawn((
                    Node {
                        width: percent(100),
                        height: percent(100),
                        min_height: px(220),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.13, 0.14, 0.17)),
                    BorderColor::all(Color::srgb(0.24, 0.26, 0.33)),
                    BorderRadius::all(px(8)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(
                            "Uses ViewportNode internally\nworld Spine renderer + UI layout",
                        ),
                        TextColor(Color::srgb(0.85, 0.88, 0.94)),
                    ));
                });
            });
        });
}

fn spawn_panel(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    skeleton: Handle<SkeletonData>,
    viewport_node: Node,
    spine_ui: SpineUiNode,
    interactive: Option<InteractiveSpine>,
    auto_size: Option<AutoSizeSpine>,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                min_height: px(220),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.11, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.24, 0.26, 0.33)),
            BorderRadius::all(px(8)),
        ))
        .with_children(|panel| {
            panel.spawn((Text::new(title), TextColor(Color::srgb(0.9, 0.9, 0.95))));
            let mut entity = panel.spawn((viewport_node, spine_ui, SpineUiSkeleton(skeleton)));
            if let Some(marker) = interactive {
                entity.insert(marker);
            }
            if let Some(marker) = auto_size {
                entity.insert(marker);
            }
        });
}

fn showcase_input(mut state: ResMut<ShowcaseState>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        state.fit = SpineUiFit::Contain;
    }
    if input.just_pressed(KeyCode::Digit2) {
        state.fit = SpineUiFit::Cover;
    }
    if input.just_pressed(KeyCode::Digit3) {
        state.fit = SpineUiFit::Fill;
    }
    if input.just_pressed(KeyCode::Digit4) {
        state.fit = SpineUiFit::None;
    }

    if input.just_pressed(KeyCode::KeyA) {
        state.auto_size_enabled = !state.auto_size_enabled;
    }
    if input.just_pressed(KeyCode::KeyT) {
        state.tint_index = (state.tint_index + 1) % 4;
    }
    if input.just_pressed(KeyCode::Space) {
        state.animation_index = (state.animation_index + 1) % 3;
    }
}

fn apply_showcase_state(
    state: Res<ShowcaseState>,
    mut ui_nodes: Query<
        (&mut SpineUiNode, Option<&AutoSizeSpine>, &SpineUiProxy),
        With<InteractiveSpine>,
    >,
    mut spine_query: Query<&mut Spine>,
) {
    if !state.is_changed() {
        return;
    }

    let tint = match state.tint_index {
        0 => Color::WHITE,
        1 => Color::srgb(1.0, 0.82, 0.78),
        2 => Color::srgb(0.78, 0.9, 1.0),
        _ => Color::srgb(0.86, 1.0, 0.8),
    };

    let animation = match state.animation_index {
        0 => "walk",
        1 => "run",
        _ => "portal",
    };

    for (mut spine_ui, auto_size, proxy) in &mut ui_nodes {
        spine_ui.fit = state.fit;
        spine_ui.tint = tint;
        if let Some(auto_size) = auto_size {
            spine_ui.auto_size = state.auto_size_enabled.then_some(auto_size.0);
        }

        if let Ok(mut spine) = spine_query.get_mut(proxy.proxy_entity) {
            let Spine(SkeletonController {
                animation_state, ..
            }) = spine.as_mut();
            let _ = animation_state.set_animation_by_name(0, animation, true);
        }
    }
}

fn enable_ui_debug_overlay(mut debug_options: ResMut<UiDebugOptions>) {
    debug_options.enabled = true;
    debug_options.show_clipped = true;
}

fn toggle_ui_debug_overlay(
    input: Res<ButtonInput<KeyCode>>,
    mut debug_options: ResMut<UiDebugOptions>,
) {
    if input.just_pressed(KeyCode::F1) {
        debug_options.toggle();
    }
}
