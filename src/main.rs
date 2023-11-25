//! Shape game.
//!
//! Mimic of Suika Game, a mimic itself of the chinese Synthetic Watermelon Game.
//!
//! See https://www.reddit.com/r/NintendoSwitch/comments/17gmk3q/the_popular_streaming_game_suika_is_a_knock_off/.

mod items;
mod spawning;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use items::PiecePhysicsDisplayPlugin;
use spawning::PieceSpawnPlugin;

#[allow(clippy::excessive_precision)]
const PHI: f32 = 1.618_033_988_749_894_848_204_586_834_365_638_118_f32;
const BOARD_WIDTH: f32 = 500.0;
const BOARD_HEIGHT: f32 = PHI * BOARD_WIDTH;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0),
            #[cfg(feature = "rapier-debug-render")]
            RapierDebugRenderPlugin::default(),
            PieceSpawnPlugin,
            PiecePhysicsDisplayPlugin,
        ))
        .add_systems(Startup, setup_board)
        .run();
}

fn setup_board(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(510.0, 10.0)),
                ..default()
            },
            ..default()
        })
        .insert(Collider::cuboid(255.0, 5.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -200.0, 0.0)))
        .insert(Restitution::coefficient(-0.5));

    for x in [-250.0, 250.0] {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 0.25, 0.75),
                    custom_size: Some(Vec2::new(10.0, 500.0)),
                    ..default()
                },
                ..default()
            })
            .insert(Collider::cuboid(5.0, 250.0))
            .insert(TransformBundle::from(Transform::from_xyz(x, 0.0, 0.0)))
            .insert(Restitution::coefficient(-0.5));
    }
}
