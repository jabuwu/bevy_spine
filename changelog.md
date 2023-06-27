# 0.6.0
- Improved premultiplied alpha support by pre-processing premultiplied textures
- Support Spine texture runtime settings
- Add `SpineSet::OnEvent`
- Fix some events getting missed
- Revamped material support
  - Custom material support (see `custom_material` example)
  - Added support for 3D meshes and materials (see `3d` example)
  - Added support for custom mesh creation (`SpineDrawer`)
- Spine meshes can now be drawn using the non-combined (simple) drawer
- `workaround_5732` no longer necessary, Bevy issue was fixed

# 0.5.0
- Update to Bevy 0.10
- Add lots of docs
- Improve asset loading
- Allow Spines to be spawned in one frame
- Add Atlas handle to `SpineTextureCreateEvent`
- No longer force textures to Nearest
