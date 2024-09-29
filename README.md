# bevy_obj

[![Crates.io](https://img.shields.io/crates/v/bevy_obj.svg)](https://crates.io/crates/bevy_obj)

A Wavefront .obj mesh asset loader plugin for the [Bevy engine](https://github.com/bevyengine/bevy)

## Usage:

Add the crate as a dependency:

*Major and Minor version number should match bevy version*

```toml
[dependencies]
bevy = "0.14"
bevy_obj = "0.14"
```

Add the plugin:

```rust
use bevy::prelude::*;
use bevy_obj::ObjPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .run();
}
```

Load the `.obj` file:

```rust
fn example_startup_system(asset_server: Res<AssetServer>) {
    // Load it as a singular mesh
    let mesh_handle = asset_server.load::<Mesh>("example.obj");

    // Load it as a scene with limited .mtl material support
    let scene_handle = asset_server.load::<Scene>("example.obj");

    // Or let bevy infer the type
    let model = PbrBundle {
        mesh: asset_server.load("example.obj"),
        ..default()
    };
}
```