use bevy::{prelude::*, window::close_on_esc};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Stacker".to_string(),
            width: 800.,
            height: 800.,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::default())
        .insert_resource(Spawner(Timer::from_seconds(1., true)))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_camera)
        .add_startup_system(setup_ground)
        .add_system(close_on_esc)
        .add_system(spawn_shapes)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn setup_ground(mut commands: Commands) {
    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(0., -250., 0.)))
        .insert(Collider::cuboid(250., 20.));
}

struct Spawner(Timer);

fn spawn_shapes(
    mut commands: Commands,
    mut spawner: ResMut<Spawner>,
    time: Res<Time>,
    // mouse_button_input: Res<Input<MouseButton>>,
    // windows: Res<Windows>,
) {
    if spawner.0.tick(time.delta()).just_finished() {
        let shape = match (time.seconds_since_startup() * 1000.) as i32 % 4 {
            0 => Collider::cuboid(30., 30.),
            1 => Collider::ball(30.),
            2 => Collider::capsule_x(30., 30.),
            _ => Collider::round_cuboid(30., 30., 0.1),
        };

        commands
            .spawn_bundle(TransformBundle::from(Transform::from_xyz(0., 100., 0.)))
            .insert(RigidBody::Dynamic)
            .insert(shape);
    }
    // if mouse_button_input.pressed(MouseButton::Left) {
    // let window = windows.primary().cursor_position();
}
