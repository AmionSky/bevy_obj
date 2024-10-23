# bevy_obj

[![Crates.io](https://img.shields.io/crates/v/bevy_obj.svg)](https://crates.io/crates/bevy_obj)

Wavefront OBJ mesh asset loader plugin for the [Bevy engine](https://github.com/bevyengine/bevy).

## Usage

Add the crate as a dependency:

**Major and Minor version number should match bevy version.**

```toml
[dependencies]
bevy = "0.15"
bevy_obj = "0.15"
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

Load an `.obj` file:

```rust
fn example_startup_system(asset_server: Res<AssetServer>) {
    // Load it as a singular mesh
    let mesh_handle = asset_server.load::<Mesh>("example.obj");

    // Load it as a scene with limited .mtl material support
    let scene_handle = asset_server.load::<Scene>("example.obj");

    // Or let bevy infer the type
    let mesh = Mesh3d(asset_server.load("example.obj"));
    let scene = SceneRoot(asset_server.load("example.obj"));
}
```

## Settings

You can use `load_with_settings()` to modify some loader settings.

```rust
fn example_startup_system(asset_server: Res<AssetServer>) {
    // Load the model with flat normals
    let scene = SceneRoot(asset_server.load_with_settings(
        "example.obj",
        |settings: &mut bevy_obj::ObjSettings| {
            settings.force_compute_normals = true;
            settings.prefer_flat_normals = true;
        },
    ));
}
```

## License

Licensed under either of

 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
