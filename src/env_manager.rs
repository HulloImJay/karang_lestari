use crate::region_sampler::{LightingSetup, RegionSampler};
use bevy::app::{App, Plugin};
use bevy::color::Srgba;
use bevy::pbr::{DirectionalLight, DistanceFog};
use bevy::prelude::{Camera3d, ClearColor, Component, GlobalTransform, Query, Res, ResMut, Resource, Time, Transform, Update, With, Without};
use glam::{FloatExt, Quat, Vec2, Vec3};
use std::f32::consts::PI;

#[derive(Default)]
pub struct EnvManagerPlugin;

impl Plugin for EnvManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnvManager {
            time_of_day: 0.65,
            time_of_day_speed: 0.003,
        })
        .add_systems(Update, env_update_system);
    }
}

#[derive(Component)]
pub struct MainLight {}

#[derive(Component)]
pub struct SecondaryLight {}

#[derive(Resource)]
pub struct EnvManager {
    time_of_day: f32,
    time_of_day_speed: f32,
}

pub fn env_update_system(
    time: Res<Time>,
    mut env_manager: ResMut<EnvManager>,
    mut query_main_light: Query<
        (&mut Transform, &mut DirectionalLight, &mut MainLight),
        Without<SecondaryLight>,
    >,
    mut query_secondary_light: Query<
        (&mut Transform, &mut DirectionalLight, &mut SecondaryLight),
        Without<MainLight>,
    >,
    mut query_cam: Query<(&GlobalTransform, &mut DistanceFog), With<Camera3d>>,
    region_sampler: Res<RegionSampler>,
    mut clear_colour: ResMut<ClearColor>,
) {
    env_manager.time_of_day =
        (env_manager.time_of_day + time.delta_secs() * env_manager.time_of_day_speed) % 1.0;

    let daytime = env_manager.time_of_day > 0.4 || env_manager.time_of_day_speed < 0.1;

    let (cam_t, mut cam_fog) = query_cam.single_mut().unwrap();
    let ([r1, r2, r3], [w1, w2, w3]) =
        region_sampler.sample_region(Vec2::new(cam_t.translation().x, cam_t.translation().z));
    let region1 = &region_sampler.regions[r1];
    let region2 = &region_sampler.regions[r2];
    let region3 = &region_sampler.regions[r3];

    let r1_lighting =
        interpolate_lighting_setups(env_manager.time_of_day, &region1.lighting_setups);
    let r2_lighting =
        interpolate_lighting_setups(env_manager.time_of_day, &region2.lighting_setups);
    let r3_lighting =
        interpolate_lighting_setups(env_manager.time_of_day, &region3.lighting_setups);

    let lighting_setup = tri_lerp_lighting([r1_lighting, r2_lighting, r3_lighting], [w1, w2, w3]);

    // primary
    for (mut transform, mut directional, mut main_light) in query_main_light.iter_mut() {
        transform.rotation = Quat::from_axis_angle(Vec3::X, env_manager.time_of_day * PI * 2.0);

        directional.shadows_enabled = daytime;
        directional.color = lighting_setup.primary_color;
        directional.illuminance = lighting_setup.primary_illuminance;
    }

    // secondary
    for (mut transform, mut directional, _) in query_secondary_light.iter_mut() {
        transform.rotation =
            Quat::from_axis_angle(Vec3::X, PI + env_manager.time_of_day * PI * 2.0);
        directional.shadows_enabled = !daytime;
        directional.color = lighting_setup.secondary_color;
        directional.illuminance = lighting_setup.secondary_illuminance;
    }

    clear_colour.0 = lighting_setup.clear_colour;
    cam_fog.color = lighting_setup.fog_colour;
}

fn interpolate_lighting_setups(t: f32, lighting_setups: &Vec<LightingSetup>) -> LightingSetup {
    let step_size = 1.0 / (lighting_setups.len() - 1) as f32;

    let mut total = 0.0;
    let mut start_ind = 0;
    let mut remainder = 0.0;
    for i in 0..lighting_setups.len() - 1 {
        start_ind = i;

        if t >= lighting_setups[i].time && t <= lighting_setups[i+1].time {
            break;
        }
    }
    let time_one = lighting_setups[start_ind].time;
    let time_two = lighting_setups[start_ind + 1].time;

    let sub_t = inv_lerp(time_one, time_two, t);

    // println!(
    //     "t: {}; total setups: {}; betwix:{}-{}; indices: {}-{}; times:{}-{}; sub_t: {}",
    //     t,
    //     lighting_setups.len(),
    //     lighting_setups[start_ind].name,
    //     lighting_setups[start_ind+1].name,
    //     start_ind,
    //     start_ind + 1,
    //     time_one,
    //     time_two,
    //     sub_t
    // );

    let setup_1 = &lighting_setups[start_ind];
    let setup_2 = &lighting_setups[start_ind + 1];

    LightingSetup {
        primary_color: lerp_srgba(
            setup_1.primary_color.to_srgba(),
            setup_2.primary_color.to_srgba(),
            sub_t,
        )
        .into(),
        primary_illuminance: setup_1
            .primary_illuminance
            .lerp(setup_2.primary_illuminance, sub_t),
        secondary_color: lerp_srgba(
            setup_1.secondary_color.to_srgba(),
            setup_2.secondary_color.to_srgba(),
            sub_t,
        )
        .into(),
        secondary_illuminance: setup_1
            .secondary_illuminance
            .lerp(setup_2.secondary_illuminance, sub_t),
        name: "dummy".into(),
        time: 0.0,
        clear_colour: lerp_srgba(
            setup_1.clear_colour.to_srgba(),
            setup_2.clear_colour.to_srgba(),
            sub_t,
        )
            .into(),
        fog_colour: lerp_srgba(
            setup_1.fog_colour.to_srgba(),
            setup_2.fog_colour.to_srgba(),
            sub_t,
        )
            .into(),
    }
}

fn tri_lerp_lighting(l: [LightingSetup; 3], w: [f32; 3]) -> LightingSetup {
    LightingSetup {
        primary_color: (l[0].primary_color.to_srgba() * w[0]
            + l[1].primary_color.to_srgba() * w[1]
            + l[2].primary_color.to_srgba() * w[2])
            .into(),
        primary_illuminance: l[0].primary_illuminance * w[0]
            + l[1].primary_illuminance * w[1]
            + l[2].primary_illuminance * w[2],
        secondary_color: (l[0].secondary_color.to_srgba() * w[0]
            + l[1].secondary_color.to_srgba() * w[1]
            + l[2].secondary_color.to_srgba() * w[2])
            .into(),
        secondary_illuminance: l[0].secondary_illuminance * w[0]
            + l[1].secondary_illuminance * w[1]
            + l[2].secondary_illuminance * w[2],
        fog_colour:(l[0].fog_colour.to_srgba() * w[0]
            + l[1].fog_colour.to_srgba() * w[1]
            + l[2].fog_colour.to_srgba() * w[2])
            .into(),
        clear_colour:(l[0].clear_colour.to_srgba() * w[0]
            + l[1].clear_colour.to_srgba() * w[1]
            + l[2].clear_colour.to_srgba() * w[2])
            .into(),
        name: "dummy".into(),
        time: 0.0,
    }
}

fn lerp_srgba(one: Srgba, two: Srgba, t: f32) -> Srgba {
    Srgba {
        red: one.red.lerp(two.red, t),
        green: one.green.lerp(two.green, t),
        blue: one.blue.lerp(two.blue, t),
        alpha: one.alpha.lerp(two.alpha, t),
    }
}

pub fn inv_lerp(a: f32, b: f32, value: f32) -> f32 {
    (value - a) / (b - a)
}
