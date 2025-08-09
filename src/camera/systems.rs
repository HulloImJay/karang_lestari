use crate::camera::components::*;
use crate::smooth_math::{smooth_damp_angle, smooth_damp_vec2, smooth_damp_vec3};
use bevy::ecs::component::ComponentId;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use crate::fishy::FishMovement;

// Spawn function—call this after you’ve spawned your target entity
pub fn spawn_camera_rig(mut commands: Commands, query_target: Query<Entity, With<FollowTarget>>) {
    let target = query_target.single();

    // 0) Root follow node
    commands
        .spawn((
            Transform::from_translation(Vec3::new(0.0, 3.5, 0.0)),
            Visibility::default(),
            SmoothFollow {
                target: target.expect("REASON"),
                smooth_time: 0.15,
                max_speed: 10.0,
                velocity: Vec3::ZERO,
            },
        ))
        .with_children(|parent| {
            // 1) Offset node (lift in Y)
            parent
                .spawn((
                    Transform::from_translation(Vec3::Y * 0.2),
                    Visibility::default(),
                ))
                .with_children(|parent| {
                    // 2) Rotation pivot
                    parent
                        .spawn((Transform::default(), Visibility::default(), SmoothOrbit {
                            mouse_sensitivity: Vec2::new(5.0, 5.0),
                            joystick_sensitivity: Vec2::new(45.0, 45.0),
                            dead_zone: 0.1,
                            smooth_time: 0.2,
                            auto_smooth_time: 3.0,
                            velocity: Vec2::ZERO,
                            angles: Vec2::new(0.0f32, 45.0f32),
                            max_speed: 720.0,
                            pitch_min: -80.0,
                            pitch_max: 80.0,
                            input_timeout : 3.0,
                            input_timeout_timer: 0.0,
                        }))
                        .with_children(|parent| {
                            // 3) Dolly offset (backwards on Z)
                            parent
                                .spawn((
                                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                    Visibility::default(),
                                ))
                                .with_children(|parent| {
                                    // 4) Actual camera
                                    parent.spawn((
                                        Camera3d::default(),
                                        DistanceFog {
                                            color: Color::srgba(0.0, 0.85, 0.90, 1.0),
                                            directional_light_color: Color::NONE,
                                            directional_light_exponent: 0.0,
                                            falloff: FogFalloff::ExponentialSquared {
                                                density: 0.08,
                                            },
                                        },
                                    ));
                                });
                        });
                });
        });
}

pub fn smooth_orbit(
    time: Res<Time>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut Transform, &mut SmoothOrbit)>,
    mut swimmer_query: Query<& Transform, (With<FishMovement>, Without<SmoothOrbit>)>,
) {
    let dt = time.delta_secs();
    let mouse_delta = mouse_motion.delta;

    for (mut tx, mut orbit) in &mut query {
        // 1) start with mouse
        let mut raw_delta = mouse_delta * orbit.mouse_sensitivity;

        // 2) add each gamepad’s right‐stick, with dead‐zone
        for gp in gamepads.iter() {
            let mut x = gp.get(GamepadAxis::RightStickX).unwrap_or(0.0);
            let mut y = gp.get(GamepadAxis::RightStickY).unwrap_or(0.0);

            // dead‐zone filter
            if x.abs() < orbit.dead_zone {
                x = 0.0;
            }
            if y.abs() < orbit.dead_zone {
                y = 0.0;
            }

            raw_delta += Vec2::new(x, -y) * orbit.joystick_sensitivity;
        }


        if raw_delta.length_squared() > f32::EPSILON {
            orbit.input_timeout_timer = orbit.input_timeout;
        }
        else {
            orbit.input_timeout_timer = (orbit.input_timeout_timer - dt).max(0.0);
        }


        // 3) compute where we want to go
        let mut target_angles =orbit.angles + raw_delta;
        let mut smooth_time = orbit.smooth_time;

        if orbit.input_timeout_timer < f32::EPSILON  {

            // with no manual input (timed out) we aim toward facing

            let mut angles = orbit.angles;
            for swim_tx in swimmer_query.iter(){
                let euler = swim_tx.rotation.to_euler(EulerRot::YXZ);
                angles = Vec2::new(euler.0.to_degrees(), euler.1.to_degrees());
            }
            target_angles = -angles;
            smooth_time = orbit.auto_smooth_time;
        };

        // 4) damp from current → target
        let (mut new_angle_x, new_vel_x) = smooth_damp_angle(
            orbit.angles.x,
            target_angles.x,
            orbit.velocity.x,
            smooth_time,
            dt,
            orbit.max_speed,
        );
        orbit.velocity.x = new_vel_x;
        let (mut new_angle_y, new_vel_y) = smooth_damp_angle(
            orbit.angles.y,
            target_angles.y,
            orbit.velocity.y,
            smooth_time,
            dt,
            orbit.max_speed,
        );
        orbit.velocity.y = new_vel_y;

        // 5) clamp pitch
        new_angle_y = new_angle_y.clamp(orbit.pitch_min, orbit.pitch_max);
        orbit.angles = Vec2::new(new_angle_x, new_angle_y);

        // 6) build rotation
        let yaw = Quat::from_rotation_y(-orbit.angles .x.to_radians());
        let pitch = Quat::from_rotation_x(-orbit.angles .y.to_radians());
        tx.rotation = yaw * pitch;
    }
}

pub fn smooth_follow(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut SmoothFollow)>,
    targets: Query<&GlobalTransform, Without<SmoothFollow>>,
) {

    for (mut tx, mut follow) in &mut query {
        if let Ok(target_tf) = targets.get(follow.target) {
            let dt = time.delta_secs();
            let goal = target_tf.translation();
            // critically damped smoothing (per-axis)
            let (new_pos, new_vel) = smooth_damp_vec3(
                tx.translation,
                goal,
                follow.velocity,
                follow.smooth_time,
                dt,
                follow.max_speed,
            );
            // info!(
            //     "dt = {:.4}, change_len = {:.4}, max_change = {:.4}, vel = {:?}",
            //     dt,
            //     (tx.translation - goal).length(),
            //     follow.max_speed * follow.smooth_time,
            //     follow.velocity
            // );
            tx.translation = new_pos;
            follow.velocity = new_vel;

            // hack
            // let goal = target_tf.translation();
            // tx.translation = goal + Vec3::new(0.0, 0.0, 1.0);
        }
    }
}
