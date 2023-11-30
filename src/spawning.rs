use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::geometry::Collider;
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::{
    pieces::{nth_piece, nth_piece_display, Piece, PieceAssets},
    MainCamera, BOARD_BOTTOM_Y, BOARD_HEIGHT, BOARD_WIDTH,
};

const SPAWN_Y: f32 = BOARD_BOTTOM_Y + BOARD_HEIGHT;
const CURSOR_Z: f32 = 15.0;

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
struct PieceWaitingOnCursor;
#[derive(Component)]
struct ItemWaiting2nd;

#[derive(Resource)]
struct GameRand(rand_pcg::Pcg64);

#[derive(Component, Debug)]
struct PieceWidth {
    left: f32,
    right: f32,
}

impl PieceWidth {
    fn from_collider(c: &Collider) -> Self {
        PieceWidth {
            left: c
                .project_point(Vec2::ZERO, 0.0, Vec2 { x: 1000.0, y: 0.0 }, true)
                .point
                .x,
            right: c
                .project_point(Vec2::ZERO, 0.0, Vec2 { x: -1000.0, y: 0.0 }, true)
                .point
                .x,
        }
    }
    fn clamp(&self, t: Vec3) -> Vec3 {
        Vec3 {
            x: t.x.clamp(
                -BOARD_WIDTH / 2.0 + self.left,
                BOARD_WIDTH / 2.0 + self.right,
            ),
            y: t.y,
            z: t.z,
        }
    }
}

fn replenish_cursor(
    time: Res<Time>,
    mut timer: ResMut<PieceAvailableTimer>,
    cursor: Query<&Transform, With<Cursor>>,
    second_piece: Query<&Piece, With<ItemWaiting2nd>>,
    second_entity: Query<Entity, With<ItemWaiting2nd>>,
    mut commands: Commands,
    assets: Res<PieceAssets>,
    mut rand: ResMut<GameRand>,
) {
    if let Ok(mut transform) = cursor.get_single().cloned() {
        if timer.0.tick(time.delta()).just_finished() {
            log::debug!("replenished cursor");
            let second_rank = match second_piece.get_single() {
                Ok(piece) => piece.rank,
                Err(_) => 0,
            };
            for e in second_entity.iter() {
                commands.entity(e).despawn_recursive();
            }
            let width = PieceWidth::from_collider(&assets.shape_colliders[second_rank]);
            transform.translation = width.clamp(transform.translation);
            transform.translation.y = SPAWN_Y;
            commands.spawn((
                nth_piece_display(&assets, second_rank, transform),
                width,
                PieceWaitingOnCursor,
            ));
            commands.spawn((
                nth_piece_display(
                    &assets,
                    rand.0.gen_range(0..=4),
                    Transform::from_translation(Vec3::new(BOARD_WIDTH / 2.0 + 80.0, 300.0, 1.0)),
                ),
                ItemWaiting2nd,
            ));
        }
    }
}

fn spawn_items(
    buttons: Res<Input<MouseButton>>,
    item_waiting: Query<(&Piece, &Transform, Entity), With<PieceWaitingOnCursor>>,
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
    mut cursor_items: Query<
        (&mut Transform, Option<&PieceWidth>),
        Or<(With<Cursor>, With<PieceWaitingOnCursor>)>,
    >,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // see https://bevy-cheatbook.github.io/cookbook/cursor2world.html
    let (camera, camera_transform) = q_camera.single();

    for event in events.read() {
        if let Some(world_position) = camera
            .viewport_to_world(camera_transform, event.position)
            .map(|ray| ray.origin.truncate())
        {
            log::debug!("new mouse position: {:?}", world_position);
            let position = Vec3 {
                x: world_position
                    .x
                    .clamp(-BOARD_WIDTH / 2.0, BOARD_WIDTH / 2.0),
                y: SPAWN_Y,
                z: CURSOR_Z,
            };
            for (mut cursor_transform, width) in cursor_items.iter_mut() {
                cursor_transform.translation = position;

                if let Some(w) = width {
                    log::debug!("width: {w:?}");
                    cursor_transform.translation.x = cursor_transform
                        .translation
                        .x
                        .clamp(-BOARD_WIDTH / 2.0 + w.left, BOARD_WIDTH / 2.0 + w.right);
                }
            }
        }
    }
}

fn populate_spawning_entities(mut commands: Commands) {
    commands.spawn((
        Cursor,
        TransformBundle::from_transform(Transform::from_translation(Vec3 {
            x: 0.0,
            y: SPAWN_Y,
            z: CURSOR_Z,
        })),
    ));
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
            .insert_resource(PieceAvailableTimer::default())
            .insert_resource(GameRand(Pcg64::seed_from_u64(0)));
    }
}
