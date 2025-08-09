use bevy::ecs::component::ComponentId;
use crate::smooth_math::smooth_damp_vec2;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

// Marker for your actual thing to follow
#[derive(Component)]
pub struct FollowTarget;

// Component on the root of your rig
#[derive(Component)]
pub struct SmoothFollow {
    pub target: Entity,
    pub smooth_time: f32,
    pub max_speed: f32,
    pub velocity: Vec3, // for critically‐damped smoothing
}

// Component to drive rotation on the pivot node
#[derive(Component)]
pub struct SmoothOrbit {
    pub mouse_sensitivity: Vec2,
    pub joystick_sensitivity: Vec2,
    pub dead_zone: f32,
    pub smooth_time: f32,
    pub auto_smooth_time: f32,
    pub max_speed: f32,
    pub velocity: Vec2,
    pub angles: Vec2, // .x = yaw, .y = pitch (in degrees)

    pub pitch_min: f32, // lowest pitch (e.g. -80°)
    pub pitch_max: f32, // highest pitch (e.g. +80°)

    pub input_timeout_timer: f32,
    pub input_timeout: f32,
}

