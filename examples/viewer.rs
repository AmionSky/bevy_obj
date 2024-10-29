use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_obj::ObjPlugin;
use std::path::Path;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (file_drop, (input, camera).chain()))
        .run();
}

fn file_drop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<FileDragAndDrop>,
    query: Query<Entity, With<SceneRoot>>,
) {
    for event in events.read() {
        if let Some(path) = drop_path(event) {
            if is_obj(path) {
                info!("Loading OBJ file: {:?}", path);

                // Despawn old OBJ
                if let Ok(scene) = query.get_single() {
                    commands.entity(scene).despawn();
                }

                // Spawn new OBJ
                commands.spawn((SceneRoot(asset_server.load(path)), Transform::IDENTITY));
            } else {
                warn!("Not an OBJ file: {:?}", path);
            }
        }
    }
}

fn drop_path(event: &FileDragAndDrop) -> Option<&Path> {
    match event {
        FileDragAndDrop::DroppedFile {
            window: _,
            path_buf: path,
        } => Some(path.as_path()),
        _ => None,
    }
}

fn is_obj(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext) = ext.to_str() {
            return ext.to_lowercase() == "obj";
        }
    }
    false
}

fn setup(mut commands: Commands) {
    info!("Drag and drop an OBJ file on the window to load it.");
    info!("RMB + Drag to rotate view");
    info!("LMB + Drag to adjust height");
    info!("Scroll to adjust view distance");

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -40.0, 60.0, 0.0)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::IDENTITY,
        ViewerCamera::default(),
    ));
}

#[derive(Component)]
struct ViewerCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub height: f32,
}

impl Default for ViewerCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance: 5.0,
            height: 0.0,
        }
    }
}

fn input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut motions: EventReader<MouseMotion>,
    mut scrolls: EventReader<MouseWheel>,
    mut query: Query<&mut ViewerCamera>,
) {
    let mut camera = query.single_mut();

    // Rotation
    if buttons.pressed(MouseButton::Left) {
        for motion in motions.read() {
            use std::f32::consts::FRAC_PI_2;

            camera.yaw -= motion.delta.x * 0.008;
            camera.pitch -= motion.delta.y * 0.008;
            camera.pitch = camera.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
        }
    }

    // Height
    if buttons.pressed(MouseButton::Right) {
        for motion in motions.read() {
            camera.height += motion.delta.y * 0.006;
        }
    }

    // Distance
    for scroll in scrolls.read() {
        camera.distance -= scroll.y * (camera.distance / 5.0).max(0.05);
        camera.distance = camera.distance.max(0.0);
    }
}

fn camera(mut query: Query<(&ViewerCamera, &mut Transform)>) {
    let (camera, mut transform) = query.single_mut();

    let rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
    let position = rotation.mul_vec3(Vec3::new(0.0, 0.0, camera.distance));
    let height = Vec3::new(0.0, camera.height, 0.0);

    transform.rotation = rotation;
    transform.translation = position + height;
}
