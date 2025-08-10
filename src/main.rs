mod camera;
mod chunked_env;
mod env_manager;
mod fishy;
mod height_noise;
mod object_manager;
mod region_sampler;
mod smooth_math;
mod turtle_model;

use crate::camera::components::FollowTarget;
use crate::camera::plugin::OrbitCameraPlugin;
use crate::camera::systems::smooth_follow;
use crate::chunked_env::ChunkedEnvironmentPlugin;
use crate::env_manager::{EnvManagerPlugin, MainLight, SecondaryLight};
use crate::fishy::{fish_movement_system, FishMovement};
use crate::height_noise::HeightNoise;
use crate::object_manager::ObjectManagerPlugin;
use crate::region_sampler::{LightingSetup, ObjectSelection, Region, RegionSampler};
use crate::turtle_model::TurtlePlugin;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use std::f32::consts::PI;
use bevy::window::WindowTheme;

#[cfg(target_arch = "wasm32")]
const _: () = {
    use getrandom02 as _;
};

fn main() {
    let standard_lights: Vec<LightingSetup> = get_standard_lights();

    let common_objects = get_common_objects();

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "Karang Lestari".into(),
                name: Some("karang_lestari".into()),
                resolution: (1080., 720.).into(),
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: true,
                    ..Default::default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_systems(
            Startup,
            (setup_env, setup_player).before(camera::systems::spawn_camera_rig),
        )
        .add_plugins(ObjectManagerPlugin::default())
        .add_plugins(TurtlePlugin::default())
        .add_plugins(OrbitCameraPlugin::default())
        .add_plugins(EnvManagerPlugin::default())
        .add_systems(FixedUpdate, fish_movement_system.before(smooth_follow))
        .insert_resource(TitleResource {
            showing: true,
            alpha: 0.0,
        })
        .add_systems(Update, title_system)
        .insert_resource(ClearColor(Color::srgb(0.2, 0.71, 0.75)))
        .insert_resource(RegionSampler::new(
            vec![
                Region::new(
                    "Smooth Sandbanks".into(),
                    4,
                    HeightNoise {
                        perlin_height: 1.0,
                        perlin_scale: 0.1,
                        perlin: Default::default(),
                        terrace_height: 2.0,
                        terrace_steps: 4.0,
                        terrace_scale: 0.02,
                        terrace_source: Default::default(),
                        terrace_smooth_width: 0.1,
                        offset: 0.0,
                    },
                    common_objects.clone(),
                    standard_lights.clone(),
                ),
                Region::new(
                    "Lil Cliffs".into(),
                    3,
                    HeightNoise {
                        perlin_height: 1.0,
                        perlin_scale: 0.1,
                        perlin: Default::default(),
                        terrace_height: 15.0,
                        terrace_steps: 3.0,
                        terrace_scale: 0.01,
                        terrace_source: Default::default(),
                        terrace_smooth_width: 0.2,
                        offset: 0.0,
                    },
                    common_objects.clone(),
                    standard_lights.clone(),
                ),
                Region::new(
                    "Big Cliffs".into(),
                    2,
                    HeightNoise {
                        perlin_height: 1.0,
                        perlin_scale: 0.1,
                        perlin: Default::default(),
                        terrace_height: 55.0,
                        terrace_steps: 4.0,
                        terrace_scale: 0.02,
                        terrace_source: Default::default(),
                        terrace_smooth_width: 0.2,
                        offset: -30.0,
                    },
                    Vec::from([
                        ObjectSelection {
                            name: "acropora_cytherea_2_komang".into(),
                            selection_weight: 1,
                        },
                        ObjectSelection {
                            name: "tendrils".into(),
                            selection_weight: 10,
                        },
                        ObjectSelection {
                            name: "small_yellow_coral_paula".into(),
                            selection_weight: 1,
                        },
                    ]),
                    vec![
                        LightingSetup {
                            name: "Depths".into(),
                            primary_color: Color::srgb(0.97, 0.5, 0.8),
                            primary_illuminance: 2_000.,
                            secondary_color: Color::srgb(0.75, 0.7, 1.0),
                            secondary_illuminance: 2_000.,
                            time: 0.0,
                            fog_colour: Color::srgba(0.0, 0.15, 0.1, 1.0),
                            clear_colour: Color::srgb(0.0, 0.15, 0.1),
                        },
                        LightingSetup {
                            name: "Depths Wrap".into(),
                            primary_color: Color::srgb(0.97, 0.5, 0.8),
                            primary_illuminance: 2_000.,
                            secondary_color: Color::srgb(0.75, 0.7, 1.0),
                            secondary_illuminance: 2_000.,
                            time: 1.0,
                            fog_colour: Color::srgba(0.0, 0.15, 0.1, 1.0),
                            clear_colour: Color::srgb(0.0, 0.15, 0.1),
                        },
                    ],
                ),
            ],
            100.0,
            0.3,
            10.0,
            42,
        ))
        .add_plugins(ChunkedEnvironmentPlugin)
        .run();
}

fn get_standard_lights() -> Vec<LightingSetup> {
    vec![
        LightingSetup {
            name: "Sunset Wrap".into(),
            primary_color: Color::srgb(0.9, 0.4, 0.7),
            primary_illuminance: 3_000.,
            secondary_color: Color::srgb(0.8, 0.5, 0.95),
            secondary_illuminance: 1_000.,
            time: 0.0,
            fog_colour: Color::srgba(0.7, 0.3, 0.1, 1.0),
            clear_colour: Color::srgb(0.8, 0.4, 0.15),
        },
        LightingSetup {
            name: "Early Night".into(),
            primary_color: Color::srgb(0.97, 0.5, 0.8),
            primary_illuminance: 500.,
            secondary_color: Color::srgb(0.75, 0.7, 1.0),
            secondary_illuminance: 2_000.,
            time: 0.1,
            fog_colour: Color::srgba(0.2, 0.1, 0.3, 1.0),
            clear_colour: Color::srgb(0.2, 0.15, 0.2),
        },
        LightingSetup {
            name: "Early Morning".into(),
            primary_color: Color::srgb(0.97, 0.7, 0.5),
            primary_illuminance: 500.,
            secondary_color: Color::srgb(0.75, 0.7, 0.95),
            secondary_illuminance: 2_000.,
            time: 0.4,
            fog_colour: Color::srgba(0.3, 0.1, 0.2, 1.0),
            clear_colour: Color::srgb(0.2, 0.15, 0.2),
        },
        LightingSetup {
            name: "Sunrise".into(),
            primary_color: Color::srgb(0.9, 0.5, 0.2),
            primary_illuminance: 3_000.,
            secondary_color: Color::srgb(0.8, 0.5, 0.95),
            secondary_illuminance: 2_000.,
            time: 0.5,
            fog_colour: Color::srgba(0.7, 0.2, 0.4, 1.0),
            clear_colour: Color::srgb(0.8, 0.3, 0.5),
        },
        LightingSetup {
            name: "Early Morning".into(),
            primary_color: Color::srgb(0.97, 0.94, 0.7),
            primary_illuminance: 6_000.,
            secondary_color: Color::srgb(0.97, 0.8, 0.8),
            secondary_illuminance: 2_000.,
            time: 0.6,
            fog_colour: Color::srgba(0.0, 0.6, 0.80, 1.0),
            clear_colour: Color::srgb(0.1, 0.5, 0.75),
        },
        LightingSetup {
            name: "Noon".into(),
            primary_color: Color::srgb(0.97, 0.94, 1.0),
            primary_illuminance: 7_000.,
            secondary_color: Color::srgb(0.97, 0.8, 0.8),
            secondary_illuminance: 3_000.,
            time: 0.75,
            fog_colour: Color::srgba(0.0, 0.85, 0.90, 1.0),
            clear_colour: Color::srgb(0.2, 0.71, 0.75),
        },
        LightingSetup {
            name: "Late Afternoon".into(),
            primary_color: Color::srgb(0.9, 0.8, 0.9),
            primary_illuminance: 6_000.,
            secondary_color: Color::srgb(0.8, 0.5, 0.95),
            secondary_illuminance: 2_000.,
            time: 0.9,
            fog_colour: Color::srgba(0.0, 0.85, 0.95, 1.0),
            clear_colour: Color::srgb(0.2, 0.71, 0.80),
        },
        LightingSetup {
            name: "Sunset".into(),
            primary_color: Color::srgb(0.9, 0.4, 0.7),
            primary_illuminance: 3_000.,
            secondary_color: Color::srgb(0.8, 0.5, 0.95),
            secondary_illuminance: 1_000.,
            time: 1.0,
            fog_colour: Color::srgba(0.7, 0.3, 0.1, 1.0),
            clear_colour: Color::srgb(0.8, 0.4, 0.15),
        },
    ]
}

fn get_common_objects() -> Vec<ObjectSelection> {
    Vec::from([
        ObjectSelection {
            name: "acropora_3_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_4_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_abrolhosensis_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_cytherea_2_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_cytherea_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_cytherea_pink_2_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_cytherea_pink_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_natalensis_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "acropora_sukarnoi_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "big_sponge_paula".into(),
            selection_weight: 1,
        },
        ObjectSelection {
            name: "diploastrea_heliopora_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "favia_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "leptoria_phrygia_2_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "leptoria_phrygia_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "lobophytum_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "mixed_coral_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "montipora_digitata_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "pincushion_starfish_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "pocillopora_meandrina_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "porites_lutea_2_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "porites_lutea_3_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "porites_lutea_4_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "porites_lutea_5_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "porites_lutea_6_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "porites_lutea_7_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "porites_lutea_anten_q".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "small_yellow_coral_paula".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "sponge_4_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "sponge_5_anten".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "sponge_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "starfish_2_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "starfish_komang".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "tendrils".into(),
            selection_weight: 5,
        },
        ObjectSelection {
            name: "turbinaria_2_anten".into(),
            selection_weight: 2,
        },
        ObjectSelection {
            name: "turbinaria_photos_komang".into(),
            selection_weight: 2,
        },
    ])
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 1. Load the GLTF scene handle
    let turtle_scene =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/sea_turtle.glb"));

    // 2. Spawn the “player” root entity
    commands
        .spawn((
            Name::new("PlayerRoot"),
            Transform::from_translation(Vec3::new(0.0, 3.5, 0.0)),
            Visibility::default(),
            FishMovement {
                acceleration: 1.0,
                deceleration: 3.0,
                lateral_deceleration: 6.0,
                max_speed: 0.66,
                sprint_multiplier: 1.5,
                velocity: Default::default(),
                current_go_force: Default::default(),
                target_direction: Default::default(),
                rotation_smooth_time: 1.5,
                go_scale_min: 0.25,
                go_scale_max: 1.0,
            },
            FollowTarget, // ← camera follow target
        ))
        .with_children(|parent| {
            // 3. Spawn the actual GLTF scene as a child
            parent.spawn((
                Name::new("FishModel"),
                SceneRoot(turtle_scene), // your SceneRoot wrapper
                Transform {
                    rotation: Quat::from_rotation_y(PI),
                    ..Default::default()
                },
                Visibility::default(),
            ));
        });

    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Photogrammetry by\nKomang Ngurah Semita Dana\nKetut Anten Wardana\nPaula Te\nMayowa Tomori\nKyle Chisholm\n\nA game by Jay"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 16.0,
            ..default()
        },
        TextShadow {
            offset: Vec2::new(1.5, 0.5),
            color: Color::linear_rgba(0.0, 0.2, 0.2, 0.75),
        },
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Right),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(15.0),
            right: Val::Px(15.0),
            ..default()
        },
    ));
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Movement: WASD or left stick\nUp/down: Q/E or X/O buttons\nCamera: mouse or right stick\nFaster (hold): space or R2\nQuit: Esc"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 16.0,
            ..default()
        },
        TextShadow {
            offset: Vec2::new(1.5, 0.5),
            color: Color::linear_rgba(0.0, 0.2, 0.2, 0.75),
        },
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Right),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));

    commands.spawn((
        Text::new("Karang Lestari"),
        TextFont {
            font: asset_server.load("fonts/Gidolinya-Regular.otf"),
            font_size: 128.0,
            ..default()
        },
        TextShadow {
            offset: Vec2::new(2.0, 2.0),
            color: Color::linear_rgba(0.2, 0.0, 0.2, 0.75),
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(150.0),
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            ..default()
        },
    ));
}

fn setup_env(mut commands: Commands) {
    // // circular base
    // commands.spawn((
    //     Mesh3d(meshes.add(Circle::new(4.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    // ));
    // // cube
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    // ));
    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            color: Color::srgb(1.0, 1.0, 1.0),
            illuminance: 6_000.,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_y(-PI / 4.) * Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        MainLight {},
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.4, 0.8, 1.0),
            illuminance: 1_000.,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_x(PI / 4.),

            ..default()
        },
        SecondaryLight {},
    ));
    // // camera
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    // ));
}

#[derive(Resource)]
struct TitleResource {
    showing: bool,
    alpha: f32,
}

fn title_system(
    time: Res<Time>,
    kb: Res<ButtonInput<KeyCode>>,
    mut title_resource: ResMut<TitleResource>,
    mut query: Query<(&mut TextColor, &mut TextShadow)>,
    mut movement_query: Query<(&mut FishMovement)>,
    mut exit: EventWriter<AppExit>,
) {
    if !title_resource.showing && kb.just_pressed(KeyCode::Escape) {
        title_resource.showing = true;
    } else if title_resource.showing && kb.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }

    let mut should_fade = false;
    for (mover) in movement_query.iter() {
        if mover.current_go_force > 0.0 {
            should_fade = true;
        }
    }

    if should_fade && title_resource.showing {
        title_resource.showing = false;
    }

    if !title_resource.showing && title_resource.alpha > 0.0 {
        title_resource.alpha = (title_resource.alpha - time.delta_secs()).max(0.0);
    }
    if title_resource.showing && title_resource.alpha < 1.0 {
        title_resource.alpha = (title_resource.alpha + time.delta_secs()).min(1.0);
    }

    for (mut text_color, mut text_shadow) in &mut query {
        let mut c = text_color.0.to_linear();
        c.alpha = title_resource.alpha;
        text_color.0 = c.into();
        c = text_shadow.color.to_linear();
        c.alpha = title_resource.alpha;
        text_shadow.color = c.into();
    }
}
