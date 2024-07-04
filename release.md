# Release Instructions

- Ensure `changelog.md` has all relevant changes from git history
- Update `readme.md`
  - Change `#[dependencies]` (both `bevy` and `bevy_spine`) to correct version
  - Update "Versions" table
- Run `cargo publish`
- Create the GitHub release
