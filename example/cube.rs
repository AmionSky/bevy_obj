use bevy::prelude::*;
use bevy_obj::ObjPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_systems(Startup, (load_mesh, load_scene, setup))
        .add_systems(Update, spin)
        .run();
}

fn load_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a spinning cube
    commands.spawn((
        PbrBundle {
            transform: Transform::from_translation(Vec3::new(-1.7, 0.0, 0.5)),
            mesh: asset_server.load("cube.obj"),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("cube.png")),
                ..default()
            }),
            ..default()
        },
        Spin,
    ));
}

fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a spinning cube
    commands.spawn((
        SceneBundle {
            transform: Transform::from_translation(Vec3::new(1.7, 0.0, -0.5)),
            scene: asset_server.load("cube.obj"),
            ..default()
        },
        Spin,
    ));
}

fn setup(mut commands: Commands) {
    // Spawn a light and the camera
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(3.0, 4.0, 3.0)),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(1.7, 2.7, 4.4))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct Spin;

fn spin(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_local_y(0.1 * time.delta_seconds());
    }
}
