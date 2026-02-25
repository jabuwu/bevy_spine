use bevy::{prelude::*, ui_render::UiDebugOptions};
use bevy_spine::{
    SkeletonData, SpinePlugin, SpineUiAnimation, SpineUiFit, SpineUiNode, SpineUiSkeleton,
};

fn main() {
    App::new()
        .insert_resource(ShowcaseState::default())
        .add_plugins((DefaultPlugins, SpinePlugin))
        .add_systems(Startup, setup)
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
    tint_index: usize,
    animation_index: usize,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            fit: SpineUiFit::Contain,
            tint_index: 0,
            animation_index: 0,
        }
    }
}

#[derive(Component)]
struct ShowcaseSpine {
    animation_offset: usize,
}

const SHOWCASE_ANIMATIONS: [&str; 3] = ["walk", "run", "portal"];

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
                min_width: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                padding: UiRect::all(px(12)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.08)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(
                    "UI Showcase (left: manual size, right: auto-size)\n[Space] next animations  [1-4] fit: contain/cover/fill/none  [T] tint all  [F1] UI borders",
                ),
                TextColor(Color::srgb(0.9, 0.92, 0.97)),
            ));

            root.spawn(Node {
                width: percent(100),
                flex_grow: 1.0,
                min_width: px(0),
                min_height: px(0),
                column_gap: px(12),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|row| {
                spawn_panel(
                    row,
                    "walk (left offset)",
                    skeleton_handle.clone(),
                    0,
                    SpineUiNode {
                        fit: SpineUiFit::Contain,
                        auto_size: None,
                        offset: Vec2::new(-90.0, -30.0),
                        scale: 1.05,
                        animation: Some(SpineUiAnimation::looping(SHOWCASE_ANIMATIONS[0])),
                        tint: Color::WHITE,
                        ..default()
                    },
                );

                spawn_panel(
                    row,
                    "run (right offset)",
                    skeleton_handle.clone(),
                    1,
                    SpineUiNode {
                        fit: SpineUiFit::Contain,
                        auto_size: Some(Vec2::new(300.0, 420.0)),
                        offset: Vec2::new(90.0, -30.0),
                        scale: 1.05,
                        animation: Some(SpineUiAnimation::looping(SHOWCASE_ANIMATIONS[1])),
                        tint: Color::WHITE,
                        ..default()
                    },
                );
            });
        });
}

fn spawn_panel(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    skeleton: Handle<SkeletonData>,
    animation_offset: usize,
    spine_ui: SpineUiNode,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(1.0)),
                border_radius: BorderRadius::all(px(8)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.11, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.24, 0.26, 0.33)),
        ))
        .with_children(|panel| {
            panel.spawn((Text::new(title), TextColor(Color::srgb(0.9, 0.9, 0.95))));
            panel.spawn((
                Node {
                    flex_grow: 1.0,
                    min_width: px(0),
                    min_height: px(0),
                    ..default()
                },
                spine_ui,
                SpineUiSkeleton(skeleton),
                ShowcaseSpine { animation_offset },
            ));
        });
}

fn showcase_input(mut state: ResMut<ShowcaseState>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Space) {
        state.animation_index = (state.animation_index + 1) % SHOWCASE_ANIMATIONS.len();
    }
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
    if input.just_pressed(KeyCode::KeyT) {
        state.tint_index = (state.tint_index + 1) % 4;
    }
}

fn apply_showcase_state(
    state: Res<ShowcaseState>,
    mut ui_nodes: Query<(&ShowcaseSpine, &mut SpineUiNode)>,
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

    for (showcase_spine, mut spine_ui) in &mut ui_nodes {
        let animation_name = SHOWCASE_ANIMATIONS
            [(state.animation_index + showcase_spine.animation_offset) % SHOWCASE_ANIMATIONS.len()];
        spine_ui.fit = state.fit;
        spine_ui.tint = tint;
        spine_ui.animation = Some(SpineUiAnimation::looping(animation_name));
    }
}

fn toggle_ui_debug_overlay(
    input: Res<ButtonInput<KeyCode>>,
    mut debug_options: ResMut<UiDebugOptions>,
) {
    if input.just_pressed(KeyCode::F1) {
        debug_options.toggle();
    }
}
