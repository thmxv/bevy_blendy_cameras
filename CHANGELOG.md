# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased] 

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.7.0]

### Changed

- Update `bevy` to `0.16`

## [0.6.1]

### Changed
- Make some raycast utilities public
- Update `bevy_egui` to `0.33`

## [0.6.0]

### Changed

- Update `bevy` to `0.15`
- Use Bevy internal raycasting facilities and move away from bevy_mod_raycast :/
- Use a configurable distance of the camera to the focus point, avoiding being 
  stuck with a distance of 0.0

### Fixed

- Pressing the fly controller keys sets the active camera, avoiding the bug
  were trying to move a camera (by flying) would move another one.

## [0.5.1]

### Changed

- Update `bevy_egui` to `0.30` and `egui` to `0.29`

### Fixed

- Fix a panic when `egui` fails to return a context

## [0.5.0]

### Changed

- Add "User" to viewpoint enum

## [0.4.1] 

### Changed

- Switch to bevy_egui 0.29
- Use system monitoring changes instead of hooks on Handle<Mesh>

## [0.4.0] 

### Added

- New egui_full example

### Changed

- Make modules private and used types public
- Events now needs the camera entity parameters to indicate which camera must
  be changed
- BlendyCamerasSystemSet refactored with different internal system order

### Fixed

- Now the Pan/Orbit/Zoom controller works with multiple viewports
- The events now suports mutliple cameras/viewports

## [0.3.1] 

### Changed

- Fixes in the README.md

## [0.3.0] 

### Added

- Documentation

### Changed

- Some functions are now private and not public anymore
- Some `use` are now private and not public anymore 

## [0.2.0] - 2024-07-26

### Changed

- Upgrade to Bevy 0.14

## [0.1.0] - 2024-07-26

### Added

- Initial release for Bevy 0.13
- Orbit camera controller allowing Pan/Orbit/Zoom with "Zoom to mouse positon" and "Auto depth"
- Fly camera controller
- Swith between camera controllers
- Set camera viewpoint (front, back, right, left, top, bottom)
- Frame view around entities
