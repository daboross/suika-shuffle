//! Shape game.
//!
//! Mimic of Suika Game, a mimic itself of the chinese Synthetic Watermelon Game.
//!
//! See https://www.reddit.com/r/NintendoSwitch/comments/17gmk3q/the_popular_streaming_game_suika_is_a_knock_off/.

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

#[derive(Resource)]
struct PieceAvailableTimer(Timer);
impl Default for PieceAvailableTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Once))
    }
}
impl PieceAvailableTimer {
    fn restart(&mut self) {
        *self = Self::default();
    }
}

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Cursor;
#[derive(Component)]
struct ItemWaitingOnCursor;
#[derive(Component)]
struct ItemWaiting2nd;

fn replenish_cursor(
    time: Res<Time>,
    mut timer: ResMut<PieceAvailableTimer>,
    cursor: Query<Entity, With<Cursor>>,
    mut commands: Commands,
    assets: Res<OurAssets>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for entity in &cursor {
            println!("adding thing as child!");
            commands
                .spawn((
                    nth_item(&assets, 0, Transform::default()),
                    ItemWaitingOnCursor,
                ))
                .insert(RigidBody::Fixed)
                .set_parent(entity);
        }
    }
}

fn spawn_items(
    buttons: Res<Input<MouseButton>>,
    item_waiting: Query<Entity, With<ItemWaitingOnCursor>>,
    mut commands: Commands,
    mut replenish_timer: ResMut<PieceAvailableTimer>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        println!("got left click");
        if let Ok(item_waiting) = item_waiting.get_single() {
            println!("found item waiting");
            commands
                .entity(item_waiting)
                .remove_parent()
                .remove::<ItemWaitingOnCursor>()
                .insert(RigidBody::Dynamic);
            replenish_timer.restart();
        }
    }
}

fn cursor_follows_cursor(
    mut events: EventReader<CursorMoved>,
    mut cursor: Query<&mut Transform, With<Cursor>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // see https://bevy-cheatbook.github.io/cookbook/cursor2world.html
    let (camera, camera_transform) = q_camera.single();

    for event in events.read() {
        if let Some(world_position) = camera
            .viewport_to_world(camera_transform, event.position)
            .map(|ray| ray.origin.truncate())
        {
            for mut cursor_transform in cursor.iter_mut() {
                cursor_transform.translation = world_position.extend(10.0);
            }
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
        ))
        .add_systems(Startup, (setup, mesh_setup))
        .add_systems(
            Update,
            (
                collision_events,
                cursor_follows_cursor,
                replenish_cursor,
                spawn_items
                    .after(cursor_follows_cursor)
                    .after(replenish_cursor),
            ),
        )
        .run();
}

#[derive(Component, Debug)]
struct MergeNumber(usize);

#[derive(Component, Debug)]
struct Merged(bool);

fn collision_events(
    mut events: EventReader<CollisionEvent>,
    info: Query<(&MergeNumber, &Transform, &Merged)>,
    mut commands: Commands,
    assets: Res<OurAssets>,
) {
    for event in events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = *event {
            if let (Ok((merge1, pos1, merged1)), Ok((merge2, pos2, merged2))) =
                (info.get(entity1), info.get(entity2))
            {
                if merge1.0 == merge2.0 && !merged1.0 && !merged2.0 {
                    commands.entity(entity1).despawn_recursive();
                    commands.entity(entity2).despawn_recursive();
                    if merge1.0 + 1 < UPGRADE_NUM {
                        let new_translation = (pos1.translation + pos2.translation) / 2.0;
                        let new_rotation = (pos1.rotation + pos2.rotation) / 2.0;
                        commands.spawn(nth_item(
                            &assets,
                            merge1.0 + 1,
                            Transform::from_translation(new_translation)
                                .with_rotation(new_rotation),
                        ));
                    }
                }
            }
        }
    }
}

#[derive(Resource, Clone)]
struct OurAssets {
    circle_mesh: Mesh2dHandle,
    circle_color_material: Handle<ColorMaterial>,
    shape_meshes: Vec<Mesh2dHandle>,
    shape_colors: Vec<Handle<ColorMaterial>>,
    shape_colliders: Vec<Collider>,
}

const MIN_RADIUS: f32 = 20.0;
const MAX_RADIUS: f32 = 200.0;
const MIN_EDGES: usize = 3;
const MAX_EDGES: usize = MIN_EDGES + UPGRADE_NUM;
const UPGRADE_NUM: usize = 6;

fn nth_radius(n: usize) -> f32 {
    MAX_RADIUS - (n - MIN_EDGES) as f32 * (MAX_RADIUS - MIN_RADIUS) / (MAX_EDGES - MIN_EDGES) as f32
}

#[test]
fn nth_behaves() {
    assert_eq!(nth_radius(MIN_EDGES), MAX_RADIUS);
    assert_eq!(nth_radius(MAX_EDGES), MIN_RADIUS);
}

impl OurAssets {
    fn init(meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>) -> Self {
        OurAssets {
            circle_mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            circle_color_material: materials.add(ColorMaterial::from(Color::PURPLE)),
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
                    Collider::convex_hull(&vertices)
                        .expect("expected calculated polygon shape to not be an almost-flat line")
                })
                .rev()
                .collect(),
        }
    }
}

fn mesh_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // // Hexagon
    // commands.spawn(MaterialMesh2dBundle {
    //     mesh: meshes.add(shape::RegularPolygon::new(50., 7).into()).into(),
    //     material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
    //     transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
    //     ..default()
    // });
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

fn circle(m: &OurAssets, transform: Transform) -> impl Bundle {
    (
        MaterialMesh2dBundle {
            mesh: m.circle_mesh.clone(),
            material: m.circle_color_material.clone(),
            transform,
            ..default()
        },
        Collider::ball(50.0),
        piece_physics(),
    )
}

struct ItemBundle {}

fn nth_item(m: &OurAssets, n: usize, transform: Transform) -> impl Bundle {
    (
        MaterialMesh2dBundle {
            mesh: m.shape_meshes[n].clone(),
            material: m.shape_colors[n].clone(),
            transform,
            ..default()
        },
        m.shape_colliders[n].clone(),
        CollidingEntities::default(),
        piece_physics(),
        Merged(false),
        MergeNumber(n.try_into().unwrap()),
        ActiveEvents::COLLISION_EVENTS,
    )
}

fn system_which_spawns(mut commands: Commands, m: Res<OurAssets>) {
    commands.spawn(circle(&m, Transform::from_xyz(0.0, 200.0, 1.0)));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let assets = OurAssets::init(&mut meshes, &mut materials);
    commands.spawn(nth_item(&assets, 1, Transform::from_xyz(0.0, 200.0, 1.0)));
    commands.spawn(nth_item(&assets, 0, Transform::from_xyz(4.0, 400.0, 1.0)));
    commands.spawn(nth_item(&assets, 0, Transform::from_xyz(4.0, 600.0, 1.0)));
    commands.spawn(nth_item(&assets, 3, Transform::from_xyz(4.0, 1000.0, 1.0)));
    commands.insert_resource(assets);

    commands.insert_resource(PieceAvailableTimer::default());
    commands.spawn((Cursor, TransformBundle::default()));
    commands.spawn((Camera2dBundle::default(), MainCamera));

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

    // Rectangle
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(50.0, 100.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
        ..default()
    });

    // Quad
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(50., 100.)).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
        ..default()
    });
}
