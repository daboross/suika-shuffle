use std::thread::spawn;

use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::{prelude::*, rapier::dynamics::RigidBodyDamping};
#[derive(Component)]
struct Person;
#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}
#[derive(Resource)]
struct GreetTimer(Timer);
impl Default for GreetTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_people)
            .init_resource::<GreetTimer>()
            .add_systems(Update, greet_people);
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0).in_fixed_schedule())
        .add_systems(Startup, (setup, mesh_setup));

    #[cfg(feature = "rapier-debug-render")]
    app.add_plugins(RapierDebugRenderPlugin::default());

    app.run();
}

#[derive(Resource, Clone)]
struct OurAssets {
    circle_mesh: Mesh2dHandle,
    circle_color_material: Handle<ColorMaterial>,
}

fn mesh_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(OurAssets {
        circle_mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        circle_color_material: materials.add(ColorMaterial::from(Color::PURPLE)),
    });
}

fn spawn_circle(m: &OurAssets, commands: &mut Commands, transform: Transform) {
    // Circle
    let insert = commands
        .spawn(MaterialMesh2dBundle {
            mesh: m.circle_mesh.clone(),
            material: m.circle_color_material.clone(),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(50.0))
        .insert(Restitution::coefficient(0.2))
        .insert(GravityScale(6.0))
        .insert(Damping {
            angular_damping: 0.0,
            linear_damping: 1.0,
        })
        .insert(TransformBundle::from(transform));
}

fn system_which_spawns(mut commands: Commands, m: Res<OurAssets>) {
    spawn_circle(&m, &mut commands, Transform::from_xyz(0.0, 200.0, 1.0));
    // spawn_circle(&m, &mut commands, Transform::from_xyz(-10.0, 150.0, 1.0));
    dbg!("hi?");
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let assets = OurAssets {
        circle_mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        circle_color_material: materials.add(ColorMaterial::from(Color::PURPLE)),
    };
    spawn_circle(&assets, &mut commands, Transform::from_xyz(0.0, 200.0, 1.0));
    spawn_circle(&assets, &mut commands, Transform::from_xyz(5.0, 400.0, 1.0));

    commands.insert_resource(assets.clone());
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(Collider::cuboid(520.0, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -200.0, 0.0)));

    commands
        .spawn(Collider::cuboid(10.0, 500.0))
        .insert(TransformBundle::from(Transform::from_xyz(-250.0, 0.0, 0.0)));
    commands
        .spawn(Collider::cuboid(10.0, 500.0))
        .insert(TransformBundle::from(Transform::from_xyz(250.0, 0.0, 0.0)));

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

    // Hexagon
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::RegularPolygon::new(50., 7).into()).into(),
        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
        transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
        ..default()
    });
}
