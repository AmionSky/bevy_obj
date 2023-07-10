# bevy_obj

[![Crates.io](https://img.shields.io/crates/v/bevy_obj.svg)](https://crates.io/crates/bevy_obj)

A Wavefront .obj mesh asset loader plugin for the [Bevy engine](https://github.com/bevyengine/bevy)

## Usage:

Add the crate as a dependency:

*Major and Minor version number should match bevy version*

```toml
[dependencies]
bevy = "0.11"
bevy_obj = "0.11"
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

Load the `.obj` file as a single mesh:

```rust
fn example_startup_system(asset_server: Res<AssetServer>) {
    let mesh_handle = asset_server.load("example.obj");
}
```

### Scene based loading

If you prefer loading `.obj` files as a scene with *(limited)* MTL material support, add the `scene` feature

```toml
[dependencies]
bevy = "0.11"
bevy_obj = { version = "0.11", features = ["scene"] }
```