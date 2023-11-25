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
                    Collider::convex_hull(&vertices)
                        .expect("expected calculated polygon shape to not be an almost-flat line")
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
    MAX_RADIUS - (n - MIN_EDGES) as f32 * (MAX_RADIUS - MIN_RADIUS) / (MAX_EDGES - MIN_EDGES) as f32
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
