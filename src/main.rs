//! Shape game.
//!
//! Mimic of Suika Game, a mimic itself of the chinese Synthetic Watermelon Game.
//!
//! See https://www.reddit.com/r/NintendoSwitch/comments/17gmk3q/the_popular_streaming_game_suika_is_a_knock_off/.

mod pieces;
mod spawning;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_rapier2d::prelude::*;
use pieces::PiecePhysicsDisplayPlugin;
use spawning::PieceSpawnPlugin;

#[allow(clippy::excessive_precision)]
const PHI: f32 = 1.618_033_988_749_894_848_204_586_834_365_638_118_f32;

const SCREEN_HEIGHT: f32 = 1000.0;
const BOARD_WIDTH: f32 = BOARD_HEIGHT / PHI;
const BOARD_HEIGHT: f32 = 800.0;
const BOTTOM_BEZEL: f32 = 50.0;
const BOARD_BOTTOM_Y: f32 = -SCREEN_HEIGHT / 2.0 + BOTTOM_BEZEL;

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .add_systems(Startup, setup_board)
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0),
            #[cfg(feature = "rapier-debug-render")]
            RapierDebugRenderPlugin::default(),
            PieceSpawnPlugin,
            PiecePhysicsDisplayPlugin,
        ))
        .run();
}

fn setup_board(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scaling_mode: ScalingMode::AutoMin {
                    min_width: BOARD_WIDTH * 1.2,
                    min_height: SCREEN_HEIGHT,
                },
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));

    commands.spawn((
        Collider::cuboid(BOARD_WIDTH / 2.0, 0.0),
        TransformBundle::from(Transform::from_xyz(0.0, BOARD_BOTTOM_Y, 0.0)),
        Restitution::coefficient(-0.5),
    ));

    for x in [-1.0, 1.0] {
        commands.spawn((
            Collider::cuboid(0.0, SCREEN_HEIGHT * 2.0),
            TransformBundle::from(Transform::from_xyz(x * BOARD_WIDTH / 2.0, 0.0, 0.0)),
            Restitution::coefficient(-0.5),
        ));
    }
}
