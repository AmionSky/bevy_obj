use bevy::prelude::*;
use bevy_obj::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_systems(Startup, (load, setup))
        .add_systems(Update, spin)
        .run();
}

#[cfg(not(feature = "scene"))]
fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a spinning cube
    commands.spawn((
        PbrBundle {
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

#[cfg(feature = "scene")]
fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a spinning cube
    commands.spawn((
        SceneBundle {
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
        transform: Transform::from_translation(Vec3::new(1.5, 2.7, 4.0))
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
