use bevy::math::Vec3;

/// Smoothly damps `current` toward `target` over time `smooth_time`.
///
/// - `current_velocity` is both an input and output: the function updates it for you.
/// - `max_speed` (optional) caps how fast you can move (pass `f32::INFINITY` to disable).
/// - Returns `(new_position, new_velocity)`.
pub fn smooth_damp_vec3(
    current: Vec3,
    target: Vec3,
    mut current_velocity: Vec3,
    smooth_time: f32,
    delta_time: f32,
    max_speed: f32,
) -> (Vec3, Vec3) {
    // Based on the numerical approach in UnityEngine.Mathf.SmoothDamp
    let smooth_time = smooth_time.max(1e-4); // avoid div by zero
    let omega = 2.0 / smooth_time;
    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    // clamp maximum speed
    let change = (current - target).clamp_length_max(max_speed * smooth_time);
    let temp_target = current - change;

    let temp_vel = (current_velocity + omega * change) * delta_time;
    current_velocity = (current_velocity - omega * temp_vel) * exp;

    let mut output = temp_target + (change + temp_vel) * exp;

    // ensure we don’t overshoot
    if (target - current).dot(output - target) > 0.0 {
        output = target;
        current_velocity = (output - target) / delta_time;
    }

    (output, current_velocity)
}

pub fn smooth_damp_f32(
    current: f32,
    target: f32,
    current_velocity: f32,
    smooth_time: f32,
    delta_time: f32,
    max_speed: f32,
) -> (f32, f32) {
    let smooth_time = smooth_time.max(1e-4);
    let omega = 2.0 / smooth_time;
    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    let change = (current - target).clamp(-max_speed * smooth_time, max_speed * smooth_time);
    let temp_target = current - change;

    let temp_vel = (current_velocity + omega * change) * delta_time;
    let mut new_velocity = (current_velocity - omega * temp_vel) * exp;

    let mut output = temp_target + (change + temp_vel) * exp;

    if (target - current) * (output - target) > 0.0 {
        output = target;
        new_velocity = 0.0;
    }

    (output, new_velocity)
}

use bevy::math::Vec2;

/// Smoothly damps `current` toward `target` over time `smooth_time` in 2D.
///
/// - `current_velocity` is both an input and output: the function updates it for you.
/// - `max_speed` caps how fast you can move (use `f32::INFINITY` to disable).
/// - Returns `(new_position, new_velocity)`.
pub fn smooth_damp_vec2(
    current: Vec2,
    target: Vec2,
    mut current_velocity: Vec2,
    smooth_time: f32,
    delta_time: f32,
    max_speed: f32,
) -> (Vec2, Vec2) {
    let smooth_time = smooth_time.max(1e-4);
    let omega = 2.0 / smooth_time;
    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    // clamp maximum speed
    let change = (current - target).clamp_length_max(max_speed * smooth_time);
    let temp_target = current - change;

    let temp_vel = (current_velocity + omega * change) * delta_time;
    current_velocity = (current_velocity - omega * temp_vel) * exp;

    let mut output = temp_target + (change + temp_vel) * exp;

    // prevent overshooting
    if (target - current).dot(output - target) > 0.0 {
        output = target;
        current_velocity = (output - target) / delta_time;
    }

    (output, current_velocity)
}

/// Smoothly damps an angular value (in degrees), taking the shortest path
/// across the 0°⇆360° boundary, using your critically-damped smooth_damp_f32.
/// Returns (new_angle, new_velocity).
pub fn smooth_damp_angle(
    current: f32,
    target: f32,
    current_velocity: f32,
    smooth_time: f32,
    delta_time: f32,
    max_speed: f32,
) -> (f32, f32) {
    // 1) Compute the raw difference and wrap into [-180, +180]
    let mut delta = (target - current) % 360.0;
    if delta < -180.0 {
        delta += 360.0;
    } else if delta > 180.0 {
        delta -= 360.0;
    }
    // 2) Build a “wrapped” target that’s always the shortest path
    let target_angle = current + delta;
    // 3) Delegate to your existing scalar smooth_damp_f32 impl
    smooth_damp_f32(
        current,
        target_angle,
        current_velocity,
        smooth_time,
        delta_time,
        max_speed,
    )
}

/// Returns the [0..1] interpolation factor t such that `lerp(a, b, t) == value`.
/// If `clamp` is `true`, it will clamp t into [0.0..1.0].
pub fn inverse_lerp(a: f32, b: f32, value: f32, clamp: bool) -> f32 {
    let t = (value - a) / (b - a);
    if clamp { t.clamp(0.0, 1.0) } else { t }
}
