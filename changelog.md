# 0.13.0
- Breaking: migrate from bundle-based spawning to Bevy required components.
  - Remove `SpineBundle`; spawn with `SkeletonDataHandle` + optional overrides.
  - Remove `SpineUiBundle`; use `SpineUiNode` with `SpineUiSkeleton` for UI.

# 0.12.0
- Add optional UI node rendering via the `ui` feature.
- Add visibility-aware mesh updates for off-screen skeletons via
  `SpineSettings::update_meshes_when_invisible` (defaults to `false`).
- Add reflection support for core components and assets.

# 0.11.0
- Update to Bevy 0.17.
- Update dependencies for Bevy 0.17 compatibility.

# 0.10.1
- No code changes, fixed version in readme

# 0.10.0
- Update to Bevy 0.14
- Fix old materials not being removed when swapping blend mode

# 0.9.0
- Upgrade runtime to Spine 4.2
- Update to `rusty_spine` 0.8
  - Add constraint APIs
  - Add physics support

# 0.8.1
- Fix dark color applying incorrectly with premultiplied alpha

# 0.8.0
- Update to Bevy 0.13

# 0.7.0
- Update to Bevy 0.12
- Add `parent` to `SpineBone`
- Rename `SpineSettings::use_3d_mesh` to `SpineSettings::mesh_type` with new `SpineMeshType` enum
- Add `Name` components to Spine mesh and bone entities
- Add a parent `SpineMeshes` entity for all `SpineMesh` entities
- Add `Debug` derive to components

# 0.6.0
- Update to Bevy 0.11
- Improve premultiplied alpha support by pre-processing premultiplied textures
- Support Spine texture runtime settings
- Fix some events getting missed, add `SpineSet::OnEvent`
- Revamp material support and settings (`SpineSettings`)
  - Custom material support (see `custom_material` example)
  - Add support for 3D meshes and materials (see `3d` example)
  - Add support for custom mesh creation (`SpineDrawer`)
- Spine meshes can now be drawn using the non-combined (simple) drawer
- `workaround_5732` no longer necessary, Bevy issue was fixed

# 0.5.0
- Update to Bevy 0.10
- Add lots of docs
- Improve asset loading
- Allow Spines to be spawned in one frame
- Add Atlas handle to `SpineTextureCreateEvent`
- No longer force textures to Nearest
