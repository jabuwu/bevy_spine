# 0.7.0
- Add `parent` to `SpineBone`
- Rename `SpineSettings::use_3d_mesh` to `SpineSettings::mesh_type` with new `SpineMeshType` enum

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
