# Bevy Blendy Cameras

## Summary

Editor like cameras controls and features inspired by Blender's viewport 
camera controls.

## Freatures

- Pan/orbit/zoom camera controls with "zoom to mouse position" and 
  "auto depth" options
- Fly camera controls
- Move to viewpoints (top, bottom, front, back, left, right)
- Frame entities into view
- Grab cursor or wrap cursor around the viewport during orbit and fly rotation
- Egui support

## TODO

- Doc
- Clean code
- Make fly mode works with orthographic projection
- Fix fly movement not working if 3D view not clicked/scrolled first
- Better multi-viewport and multi-window support
- Option for grab/wrap around of cursor

## Cargo Features

- `bevy_egui` (optional): Ignore input when `egui` had the focus

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
