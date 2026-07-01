use bevy::{
    camera::{ClearColorConfig, RenderTarget, visibility::RenderLayers},
    prelude::*,
    render::render_resource::TextureFormat,
    ui::widget::ViewportNode,
};
use bevy_spine::{SkeletonData, SkeletonDataHandle, Spine, SpinePlugin, SpineReadyEvent, SpineSet};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpinePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, on_spine_ready.in_set(SpineSet::OnReady))
        .add_systems(Update, sync_spine_viewport)
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut skeletons: ResMut<Assets<SkeletonData>>,
) {
    commands.spawn(Camera2d);

    let skeleton_handle = skeletons.add(SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    ));
    let image_handle = images.add(Image::new_target_texture(
        512,
        512,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));
    let render_layers = RenderLayers::none().with(1);

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
        ))
        .id();

    commands.spawn((
        SkeletonDataHandle(skeleton_handle),
        render_layers,
        // Keep the proxy visible. RenderLayers keep it out of the main window camera.
    ));

    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(16),
                padding: UiRect::all(px(24)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.055, 0.06, 0.08)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Spine rendered in a Bevy UI node"),
                TextColor(Color::srgb(0.96, 0.97, 1.0)),
            ));

            root.spawn((
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_width: px(0),
                    min_height: px(0),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(16)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.02, 0.025, 0.035)),
                BorderColor::all(Color::srgb(0.32, 0.36, 0.48)),
                ViewportNode::new(camera_entity),
            ));

            root.spawn((
                Text::new(
                    "The UI node owns the final size. An offscreen camera renders Spineboy into a texture, and ViewportNode displays that texture here.",
                ),
                TextColor(Color::srgb(0.68, 0.72, 0.82)),
            ));
        });
}

fn on_spine_ready(mut ready_events: MessageReader<SpineReadyEvent>, mut spines: Query<&mut Spine>) {
    for event in ready_events.read() {
        let Ok(mut spine) = spines.get_mut(event.entity) else {
            continue;
        };

        let _ = spine.animation_state.set_animation_by_name(0, "walk", true);
    }
}

fn sync_spine_viewport(
    viewport: Single<&ComputedNode, With<ViewportNode>>,
    mut spines: Query<(&mut Transform, &Spine)>,
) {
    let available_size = viewport.size();
    if available_size.x <= 1.0 || available_size.y <= 1.0 {
        return;
    }

    let Ok((mut transform, spine)) = spines.single_mut() else {
        return;
    };

    let data = spine.skeleton.data();
    let setup_size = Vec2::new(data.width(), data.height()).max(Vec2::ONE);
    let setup_center = Vec2::new(data.x(), data.y()) + setup_size * 0.5;
    let scale = (available_size / setup_size).min_element();

    *transform = Transform::from_translation((-setup_center * scale).extend(0.0))
        .with_scale(Vec3::new(scale, scale, 1.0));
}
