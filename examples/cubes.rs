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
        Mesh3d(asset_server.load("cube.obj")),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("cube.png")),
            ..default()
        })),
        Transform::from_xyz(-1.7, 0.0, 0.5),
        Spin,
    ));
}

fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a spinning cube
    commands.spawn((
        SceneRoot(asset_server.load("cube.obj")),
        Transform::from_xyz(1.7, 0.0, -0.5),
        Spin,
    ));
}

fn setup(mut commands: Commands) {
    // Spawn a light and the camera
    commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 4.0, 3.0)));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.7, 2.7, 4.4).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component)]
struct Spin;

fn spin(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_local_y(0.1 * time.delta_secs());
    }
}
