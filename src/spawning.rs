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
