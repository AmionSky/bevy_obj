# bevy_obj

[![Crates.io](https://img.shields.io/crates/v/bevy_obj.svg)](https://crates.io/crates/bevy_obj)

A Wavefront .obj mesh asset loader plugin for the [Bevy engine](https://github.com/bevyengine/bevy)

## Usage:

*Major and Minor version number should match bevy version*

Add the plugin

```rust
use bevy::prelude::*;
use bevy_obj::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .run();
}
```

Load an obj file

```rust
fn example_startup_system(asset_server: Res<AssetServer>) {
    // Load OBJ file
    let mesh_handle = asset_server.load("example.obj");
}
```