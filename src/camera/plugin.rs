// src/camera/plugin.rs
use bevy::prelude::*;
use bevy::transform::systems::{propagate_parent_transforms, sync_simple_transforms};
use crate::camera::components::{FollowTarget, SmoothOrbit};
use crate::camera::systems::{spawn_camera_rig, smooth_orbit, smooth_follow};
use crate::fishy::{fish_movement_system, FishMovement};

/// A simple plugin that handles camera‐rig spawning and its follow/orbit logic.
pub struct OrbitCameraPlugin;

impl Default for OrbitCameraPlugin {
    fn default() -> Self {
        OrbitCameraPlugin
    }
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        // Optional: register your component types for reflection/inspector, if you need it
        app
            // Spawn the camera rig once the world is ready
            .add_systems(Startup, spawn_camera_rig)
            // Per‐frame follow & orbit
            .add_systems(FixedPostUpdate, smooth_follow.after(TransformSystem::TransformPropagate))
            .add_systems(FixedUpdate, smooth_orbit)
            // .add_systems(Update, propagate_parent_transforms
            //     .after(smooth_follow))
            // .add_systems(Update, sync_simple_transforms.before(smooth_follow))
        ;
    }
}
