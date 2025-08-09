use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};

use bevy::prelude::*;
use crate::fishy::FishMovement;
use crate::height_noise::HeightNoise;
use crate::smooth_math::smooth_damp_f32;

static ANIMATION_GRAPH_PATH: &str = "animation_graphs/turtle_animations.animgraph.ron";

/// The indices of the nodes containing animation clips in the graph.
static CLIP_NODE_INDICES: [u32; 2] = [1, 2];

pub struct TurtlePlugin;

impl Default for TurtlePlugin {
    fn default() -> Self {
        TurtlePlugin
    }
}

impl Plugin for TurtlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, load_animation)
            .add_systems(Update, (init_animations, turtle_animation_system))
        ;
    }
}

#[derive(Component)]
struct TurtleAnimation {
    swim_weight_current: f32,
    swim_weight_velocity: f32,

}

#[derive(Clone, Resource)]
pub struct TurtleAnimationGraph(Handle<AnimationGraph>);

pub fn load_animation(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
){
    commands.insert_resource(TurtleAnimationGraph(
        asset_server.load(ANIMATION_GRAPH_PATH),
    ));
}

pub fn init_animations(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationPlayer)>,
    animation_graph: Res<TurtleAnimationGraph>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    for (entity, mut player) in query.iter_mut() {
        commands.entity(entity).insert((
            AnimationGraphHandle(animation_graph.0.clone()),
            TurtleAnimation{ swim_weight_current: 0.0, swim_weight_velocity: 0.0 },
        ));
        for &node_index in &CLIP_NODE_INDICES {
            player.play(node_index.into()).repeat();
        }

        *done = true;
    }
}

pub fn turtle_animation_system(
    mut movement_query: Query<(&mut FishMovement)>,
    mut anim_query: Query<(&mut AnimationPlayer, &mut TurtleAnimation)>,
    time: Res<Time>,
) {
    for (mut animation_player, mut turtle_anim) in anim_query.iter_mut() {
        for ( mover) in movement_query.iter() {

            let swim_weight = if mover.current_go_force > 0.0 { 1.0 } else { 0.0 };
            let swim_speed = mover.current_go_force;

            if swim_weight > turtle_anim.swim_weight_current {
                // quick smooth for speed up
                (turtle_anim.swim_weight_current, turtle_anim.swim_weight_velocity) = smooth_damp_f32(
                    turtle_anim.swim_weight_current,
                    swim_weight,
                    turtle_anim.swim_weight_velocity,
                    0.1,
                    time.delta_secs(),
                    100.0,
                );
            }
            else {
                // slower smooth for slow down
                (turtle_anim.swim_weight_current, turtle_anim.swim_weight_velocity) = smooth_damp_f32(
                    turtle_anim.swim_weight_current,
                    swim_weight,
                    turtle_anim.swim_weight_velocity,
                    0.7,
                    time.delta_secs(),
                    100.0,
                );
            }

            if let Some(active_animation) =
                animation_player.animation_mut(2.into())
            {
                active_animation.set_weight(turtle_anim.swim_weight_current);
                active_animation.set_speed(swim_speed);
            }

        }
    }
}