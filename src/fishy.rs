use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};

use crate::region_sampler::RegionSampler;
use bevy::prelude::*;

#[derive(Component)]
pub struct FishMovement {
    /// How quickly we speed up (units/sec²)
    pub acceleration: f32,
    /// How quickly we slow to a stop when there is no input (units/sec²)
    pub deceleration: f32,
    pub lateral_deceleration: f32,
    /// Top cruising speed (units/sec)
    pub max_speed: f32,
    /// Multiplier for sprinting
    pub sprint_multiplier: f32,
    /// Current velocity in world‐space
    pub velocity: Vec3,
    pub target_direction: Vec3,
    pub current_go_force: f32,
    /// How quickly the fish “turns” (critically-damped seconds)
    pub rotation_smooth_time: f32,
    pub go_scale_min: f32,
    pub go_scale_max: f32,
}

const STICK_DEAD_ZONE: f32 = 0.15;

fn apply_radial_deadzone(raw: Vec2, dead_zone: f32) -> Vec2 {
    let mag = raw.length();
    if mag < dead_zone {
        Vec2::ZERO
    } else {
        let norm = raw / mag;
        // remap [dead_zone..1.0] → [0.0..1.0]
        let t = (mag - dead_zone) / (1.0 - dead_zone);
        norm * t.clamp(0.0, 1.0)
    }
}

pub fn fish_movement_system(
    time: Res<Time>,
    kb: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut Transform, &mut FishMovement)>,
    // get camera’s global transform to derive forward/pan axes:
    cam_tf: Query<&GlobalTransform, With<Camera3d>>,
    region_sampler: Res<RegionSampler>,
) {
    let dt = time.delta_secs();
    let cam = cam_tf.single().unwrap();
    // camera’s *actual* forward (bevy default forward is -Z)
    let cam_forward = cam.rotation() * -Vec3::Z;
    // strafing should ignore pitch: get horizontal right
    let cam_right = cam_forward.cross(Vec3::Y).normalize();

    // build a 2D input vector from keys + left stick
    let mut inp = Vec3::ZERO;
    if kb.pressed(KeyCode::KeyW) || kb.pressed(KeyCode::ArrowUp) {
        inp.y += 1.0;
    }
    if kb.pressed(KeyCode::KeyS) || kb.pressed(KeyCode::ArrowDown) {
        inp.y -= 1.0;
    }
    if kb.pressed(KeyCode::KeyA) || kb.pressed(KeyCode::ArrowLeft) {
        inp.x -= 1.0;
    }
    if kb.pressed(KeyCode::KeyD) || kb.pressed(KeyCode::ArrowRight) {
        inp.x += 1.0;
    }

    // clamp so diagonal isn’t faster
    if inp.length() > 1.0 {
        inp = inp.normalize();
    }

    if kb.pressed(KeyCode::KeyQ) || kb.pressed(KeyCode::SuperRight) {
        inp.z -= 1.0;
    }
    if kb.pressed(KeyCode::KeyE) || kb.pressed(KeyCode::AltRight) {
        inp.z += 1.0;
    }

    // add left‐stick (each connected pad)
    for gp in gamepads.iter() {
        let raw = Vec2::new(
            gp.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
            gp.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
        );
        // invert Y so up on stick → +Y input
        let stick = Vec2::new(raw.x, raw.y);
        let stick_radial = apply_radial_deadzone(stick, STICK_DEAD_ZONE);
        inp.x += stick_radial.x;
        inp.y += stick_radial.y;

        let btn = GamepadButton::South;
        if gp.pressed(btn) {
            inp.z += 1.0;
        }
        let btn = GamepadButton::East;
        if gp.pressed(btn) {
            inp.z -= 1.0;
        }
    }

    // 1) Determine if we’re sprinting
    let mut sprint = kb.pressed(KeyCode::Space);
    for gp in gamepads.iter() {
        let btn = GamepadButton::RightTrigger2;
        if gp.pressed(btn) {
            sprint = true;
            break;
        }
    }

    for (mut tx, mut fish_movement) in &mut query {
        let speed = fish_movement.max_speed
            * if kb.pressed(KeyCode::Tab) {
                10.0
            } else if sprint {
                fish_movement.sprint_multiplier
            } else {
                1.0
            };

        // 2) Calculate the *desired* velocity in world‐space
        let desired_vel = if inp == Vec3::ZERO {
            Vec3::ZERO
        } else {
            // combine forward (with pitch) for Y, and horizontal strafing
            let dir3 = (cam_forward * inp.y + cam_right * inp.x + Vec3::Y * inp.z).normalize();
            dir3 * speed
        };

        fish_movement.current_go_force = if inp == Vec3::ZERO {
            0.0
        } else if sprint {
            2.0
        } else {
            1.0
        };

        // 5) Smoothly rotate toward the *target* direction
        if desired_vel.length_squared() > 1e-6 {
            // a) compute target direction (unit)
            fish_movement.target_direction = desired_vel.normalize();
        } else {
            fish_movement.target_direction = tx.forward().into();
        }

        // b) build a temp Transform so we can call look_at()
        let mut tmp = Transform::default();
        tmp.look_at(fish_movement.target_direction, Vec3::Y);
        let target_rot = tmp.rotation;

        // c) compute an exponential smoothing factor in [0,1]
        //    such that small delta_secs = slow start,
        //    larger dt gives faster catchup,
        //    and smooth_time is the 63%-to-target time constant.
        let t = 1.0 - (-dt / fish_movement.rotation_smooth_time).exp();

        // d) slerp current→target by that factor
        tx.rotation = tx.rotation.slerp(target_rot, t);

        let go_dot = fish_movement.target_direction.dot(tx.forward().into());

        let go_0_1 = go_dot * 0.5 + 0.5;

        let go_scale = bevy::prelude::FloatExt::lerp(fish_movement.go_scale_min, 1.0, go_0_1);

        // // 3) Accelerate / decelerate
        // // let diff = desired_vel - fish_movement.velocity;
        // // if no input, use deceleration; otherwise acceleration
        // let accel_rate = if desired_vel.length_squared() < 0.001 {
        //     fish_movement.deceleration
        // } else {
        //     fish_movement.acceleration * go_scale
        // };
        // // limit how much we can change velocity this frame

        // let delta_v = tx.forward().clamp_length_max(accel_rate * dt);
        // fish_movement.velocity += delta_v;

        if desired_vel.length_squared() > 0.01 {
            let accel_rate = fish_movement.acceleration * go_scale;
            let delta_v = tx.forward().clamp_length_max(accel_rate * dt);
            fish_movement.velocity += delta_v;

            // hack
            // fish_movement.velocity = tx.forward() * speed * go_scale
        } else {
            let diff = desired_vel - fish_movement.velocity;
            let delta_v = diff.clamp_length_max(fish_movement.deceleration * dt);
            fish_movement.velocity += delta_v;

            // hack
            // fish_movement.velocity = Vec3::ZERO;
        }

        // dampen lateral
        if fish_movement.velocity.length_squared() > 0.01 {
            let fwd_vel = fish_movement.velocity.project_onto(tx.forward().into());

            // neg of all lateral movement
            let diff = fwd_vel - fish_movement.velocity;
            let delta_v = diff.clamp_length_max(fish_movement.lateral_deceleration * dt);
            fish_movement.velocity += delta_v;
        }

        // clamp it
        fish_movement.velocity = fish_movement
            .velocity
            .clamp_length_max(speed * fish_movement.sprint_multiplier);

        // 4) Apply motion
        tx.translation += fish_movement.velocity * dt;

        // info!(
        //         "dt = {:.4}, fish_movement.velocity = {:.4}, fish_movement.velocity.length = {:.4}",
        //         dt,
        //         fish_movement.velocity,
        //         fish_movement.velocity.length(),
        //     );

        let pos_2d = Vec2::new(tx.translation.x, tx.translation.z);
        let height = region_sampler.sample_surface_height(pos_2d) as f32;
        if tx.translation.y < height + 0.3 {
            tx.translation.y = height + 0.3;
        }
        if tx.translation.y > 12.0 {
            tx.translation.y = 12.0;
        }
    }
}
