# Bevy Blendy Cameras

## Summary

Editor like cameras controls and features inspired by Blender's viewport 
camera controls.

## Features

- Pan/orbit/zoom camera controls with "zoom to mouse position" and 
  "auto depth" options
- Fly camera controls
- Move to viewpoints (top, bottom, front, back, left, right)
- Frame entities into view
- Grab cursor or wrap cursor around the viewport during orbit and fly rotation
- Egui support

## TODO

- Clean code
- Make fly mode works with orthographic projection
- Better multi-viewport and multi-window support
- Option for grab/wrap around of cursor
- Support "Auto depth" for pan movement. So that the result of the raycast under the mouse coursor always stays under the mouse cursor during pan, if no cursor grab/wrap (which there always is at the moment).
- Make the raycast used for "Auto depth" respect the materials backface culling setting. Maybe optionally because this might conflict with other raycast options used for other uses by the user.

## Default Controls

### OrbitCameraController

- Middle mouse drag - Orbit
- Shift + Middle mouse drag - Pan
- Scroll wheel - Zoom

### FlyCameraController

- Middle mouse drag - Rotate
- Scroll wheel - Change movement speed
- E - Move forward (zoom)
- D - Move backward (unzoom)
- S - Move to the left
- F - Move to the right
- W - Move to the bottom
- R - Move to the top

## Quick Start

Add the plugin:
```rust ignore
.add_plugins(BlendyCamerasPlugin)
```

Add the controllers components to a camera:
``` rust ignore
commands.spawn((
    Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        ..default()
    },
    OrbitCameraController::default(),
    FlyCameraController {
        is_enabled: false,
        ..default()
    },
));
```

Check out the [basic example](https://github.com/thmxv/bevy_blendy_cameras/tree/master/examples/basic.rs] 
to see more functionalities.

## Cargo Features

- `bevy_egui` (optional): Ignore input when `egui` has the focus

## Version Compatibility

| bevy | bevy_blendy_cameras |
|------|---------------------|
| 0.14 | 0.2                 |
| 0.13 | 0.1                 |

## Credits

- [bevy_panorbit_camera](https://github.com/Plong/bevy_panorbit_camera): The 
code for this plugin is based on this.

## Disclaimer

I am a bit new to both Rust and Bevy and this plugin is in early stages. Help 
is welcomed.
