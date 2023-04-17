use bevy::prelude::*;
use bevy_obj::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_startup_system(setup)
        .add_system(spin)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    #[cfg(not(feature = "scene"))] mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a spinning cube
    #[cfg(not(feature = "scene"))]
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

    #[cfg(feature = "scene")]
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("cube.obj"),
            ..default()
        },
        Spin,
    ));

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
