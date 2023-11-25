//! Shape game.
//!
//! Mimic of Suika Game, a mimic itself of the chinese Synthetic Watermelon Game.
//!
//! See https://www.reddit.com/r/NintendoSwitch/comments/17gmk3q/the_popular_streaming_game_suika_is_a_knock_off/.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use items::PiecePhysicsDisplayPlugin;
use spawning::PieceSpawnPlugin;

#[allow(clippy::excessive_precision)]
const PHI: f32 = 1.618_033_988_749_894_848_204_586_834_365_638_118_f32;
const BOARD_WIDTH: f32 = 500.0;
const BOARD_HEIGHT: f32 = PHI * BOARD_WIDTH;

mod spawning {
    use bevy::prelude::*;

    use crate::items::{nth_piece, nth_piece_display, Piece, PieceAssets};

    #[derive(Resource)]
    struct PieceAvailableTimer(Timer);

    impl Default for PieceAvailableTimer {
        fn default() -> Self {
            Self(Timer::from_seconds(0.5, TimerMode::Once))
        }
    }
    impl PieceAvailableTimer {
        fn restart(&mut self) {
            *self = Self::default();
        }
    }

    #[derive(Component)]
    struct Cursor;
    #[derive(Component)]
    struct ItemWaitingOnCursor;
    #[derive(Component)]
    struct ItemWaiting2nd;

    /// Used to help identify our main camera
    #[derive(Component)]
    struct MainCamera;

    fn replenish_cursor(
        time: Res<Time>,
        mut timer: ResMut<PieceAvailableTimer>,
        cursor: Query<&Transform, With<Cursor>>,
        second_piece: Query<&Piece, With<ItemWaiting2nd>>,
        second_entity: Query<Entity, With<ItemWaiting2nd>>,
        mut commands: Commands,
        assets: Res<PieceAssets>,
    ) {
        if let Ok(transform) = cursor.get_single() {
            if timer.0.tick(time.delta()).just_finished() {
                log::debug!("replenished cursor");
                let second_rank = match second_piece.get_single() {
                    Ok(piece) => piece.rank,
                    Err(_) => 0,
                };
                for e in second_entity.iter() {
                    commands.entity(e).despawn_recursive();
                }
                commands.spawn((
                    nth_piece_display(&assets, second_rank, *transform),
                    ItemWaitingOnCursor,
                ));
                commands.spawn((
                    nth_piece_display(
                        &assets,
                        second_rank,
                        Transform::from_translation(Vec3::new(300.0, 300.0, 0.0)),
                    ),
                    ItemWaiting2nd,
                ));
            }
        }
    }

    fn spawn_items(
        buttons: Res<Input<MouseButton>>,
        item_waiting: Query<(&Piece, &Transform, Entity), With<ItemWaitingOnCursor>>,
        mut commands: Commands,
        mut replenish_timer: ResMut<PieceAvailableTimer>,
        assets: Res<PieceAssets>,
    ) {
        if buttons.just_pressed(MouseButton::Left) {
            log::debug!("got left click");
            if let Ok((piece, transform, item_waiting)) = item_waiting.get_single() {
                log::debug!("spawning!");
                commands.entity(item_waiting).despawn_recursive();
                commands.spawn(nth_piece(&assets, piece.rank, *transform));
                replenish_timer.restart();
            }
        }
    }

    fn cursor_follows_cursor(
        mut events: EventReader<CursorMoved>,
        mut cursor_items: Query<&mut Transform, Or<(With<Cursor>, With<ItemWaitingOnCursor>)>>,
        q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ) {
        // see https://bevy-cheatbook.github.io/cookbook/cursor2world.html
        let (camera, camera_transform) = q_camera.single();

        for event in events.read() {
            if let Some(world_position) = camera
                .viewport_to_world(camera_transform, event.position)
                .map(|ray| ray.origin.truncate())
            {
                for mut cursor_transform in cursor_items.iter_mut() {
                    cursor_transform.translation = world_position.extend(10.0);
                    cursor_transform.translation.y = 275.0;
                }
            }
        }
    }

    fn populate_spawning_entities(mut commands: Commands) {
        commands.spawn((Cursor, TransformBundle::default()));
        commands.spawn((Camera2dBundle::default(), MainCamera));
    }

    pub struct PieceSpawnPlugin;
    impl Plugin for PieceSpawnPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, populate_spawning_entities)
                .add_systems(
                    Update,
                    (
                        cursor_follows_cursor,
                        replenish_cursor,
                        spawn_items
                            .after(cursor_follows_cursor)
                            .after(replenish_cursor),
                    ),
                )
                .insert_resource(PieceAvailableTimer::default());
        }
    }
}

mod items {
    use std::f32::consts::PI;

    use bevy::{
        prelude::*,
        sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    };
    use bevy_rapier2d::prelude::*;

    const MIN_RADIUS: f32 = 20.0;
    const MAX_RADIUS: f32 = 200.0;
    const MIN_EDGES: usize = 3;
    const MAX_EDGES: usize = MIN_EDGES + UPGRADE_NUM;
    const UPGRADE_NUM: usize = 6;

    #[derive(Component, Debug)]
    pub struct Piece {
        pub rank: usize,
    }

    #[derive(Component, Debug)]
    pub struct PieceMerged(bool);

    #[derive(Resource, Clone)]
    pub struct PieceAssets {
        shape_meshes: Vec<Mesh2dHandle>,
        shape_colors: Vec<Handle<ColorMaterial>>,
        shape_colliders: Vec<Collider>,
    }

    impl PieceAssets {
        fn init(meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>) -> Self {
            PieceAssets {
                shape_meshes: (MIN_EDGES..=MAX_EDGES)
                    // .filter(|n| n % 2 != 0)
                    .map(|edges| {
                        meshes
                            .add(shape::RegularPolygon::new(nth_radius(edges), edges).into())
                            .into()
                    })
                    .rev()
                    .collect(),
                shape_colors: [
                    Color::TURQUOISE,
                    Color::RED,
                    Color::AQUAMARINE,
                    Color::CYAN,
                    Color::BISQUE,
                    Color::BEIGE,
                    Color::ANTIQUE_WHITE,
                ]
                .into_iter()
                .map(|c| materials.add(ColorMaterial::from(c)))
                .rev()
                .collect(),
                shape_colliders: (MIN_EDGES..=MAX_EDGES)
                    // .filter(|n| n % 2 != 0)
                    .map(|edges| {
                        // see https://stackoverflow.com/a/7198179
                        // this should mirror the calculations done by `shape::RegularPolygon`
                        let radius = nth_radius(edges);

                        let vertices = (0..edges)
                            .map(|vertex| Vect {
                                x: radius * (2.0 * PI * (vertex as f32) / edges as f32).sin(),
                                y: radius * (2.0 * PI * (vertex as f32) / edges as f32).cos(),
                            })
                            .collect::<Vec<_>>();
                        Collider::convex_hull(&vertices).expect(
                            "expected calculated polygon shape to not be an almost-flat line",
                        )
                    })
                    .rev()
                    .collect(),
            }
        }
    }

    fn setup_piece_assets(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        let assets = PieceAssets::init(&mut meshes, &mut materials);
        commands.insert_resource(assets);
    }

    fn nth_radius(n: usize) -> f32 {
        MAX_RADIUS
            - (n - MIN_EDGES) as f32 * (MAX_RADIUS - MIN_RADIUS) / (MAX_EDGES - MIN_EDGES) as f32
    }

    #[test]
    fn test_nth_radius() {
        assert_eq!(nth_radius(MIN_EDGES), MAX_RADIUS);
        assert_eq!(nth_radius(MAX_EDGES), MIN_RADIUS);
    }

    fn piece_physics() -> impl Bundle {
        (
            RigidBody::Dynamic,
            Restitution::coefficient(0.8),
            GravityScale(6.0),
            Damping {
                angular_damping: 0.0,
                linear_damping: 1.0,
            },
        )
    }

    pub fn nth_piece_display(assets: &PieceAssets, n: usize, transform: Transform) -> impl Bundle {
        (
            MaterialMesh2dBundle {
                mesh: assets.shape_meshes[n].clone(),
                material: assets.shape_colors[n].clone(),
                transform,
                ..default()
            },
            Piece {
                rank: n.try_into().unwrap(),
            },
        )
    }

    pub fn nth_piece_physics(assets: &PieceAssets, n: usize) -> impl Bundle {
        (
            assets.shape_colliders[n].clone(),
            piece_physics(),
            PieceMerged(false),
            ActiveEvents::COLLISION_EVENTS,
        )
    }

    pub fn nth_piece(assets: &PieceAssets, n: usize, transform: Transform) -> impl Bundle {
        (
            nth_piece_display(assets, n, transform),
            nth_piece_physics(assets, n),
        )
    }

    fn collision_events(
        mut events: EventReader<CollisionEvent>,
        mut info: Query<(&Piece, &Transform, &mut PieceMerged)>,
        mut commands: Commands,
        assets: Res<PieceAssets>,
    ) {
        for event in events.read() {
            if let CollisionEvent::Started(entity1, entity2, _) = *event {
                let mut merged = false;
                if let (Ok((piece1, pos1, merged1)), Ok((piece2, pos2, merged2))) =
                    (info.get(entity1), info.get(entity2))
                {
                    if piece1.rank == piece2.rank && !merged1.0 && !merged2.0 {
                        merged = true;
                        commands.entity(entity1).despawn_recursive();
                        commands.entity(entity2).despawn_recursive();
                        if piece1.rank + 1 < UPGRADE_NUM {
                            let new_translation = (pos1.translation + pos2.translation) / 2.0;
                            let new_rotation = (pos1.rotation + pos2.rotation) / 2.0;
                            commands.spawn(nth_piece(
                                &assets,
                                piece1.rank + 1,
                                Transform::from_translation(new_translation)
                                    .with_rotation(new_rotation),
                            ));
                        }
                    }
                }
                if merged {
                    info.get_mut(entity1).unwrap().2 .0 = true;
                    info.get_mut(entity2).unwrap().2 .0 = true;
                }
            }
        }
    }

    pub struct PiecePhysicsDisplayPlugin;

    impl Plugin for PiecePhysicsDisplayPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, setup_piece_assets)
                .add_systems(Update, collision_events);
        }
    }
}

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
