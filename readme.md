# bevy_spine

A Bevy Plugin for [Spine](http://esotericsoftware.com/), utilizing [rusty_spine](https://github.com/jabuwu/rusty_spine). WASM compatible!

```
[dependencies]
bevy = "0.17"
bevy_spine = "0.12"
```

[See online demos!](https://jabuwu.github.io/bevy_spine_demos/) ([source repo](https://github.com/jabuwu/bevy_spine_demos))

## UI Rendering

`bevy_spine` now includes optional UI node rendering through the `ui` feature.

```toml
[dependencies]
bevy_spine = { version = "0.12", features = ["ui"] }
```

`SpineUiBundle` renders a Spine instance into a `ViewportNode`, so it can live inside Bevy UI layout.

Example:

- `cargo run --example ui_spine_showcase --features ui`

## New in 0.12

- Optional UI node rendering via `SpineUiBundle` and the `ui` feature.
- Visibility-aware mesh updates for off-screen skeletons (`SpineSettings::update_meshes_when_invisible`, defaults to `false` for optimized behavior).
- Reflection support for core components and assets (`register_type` / `register_asset_reflect`) for inspector-friendly tooling.

## Versions

| bevy_spine | rusty_spine | bevy | spine |
| ---------- | ----------- | ---- | ----- |
| 0.11, 0.12 | 0.8         | 0.17 | 4.2   |
| 0.10       | 0.8         | 0.14 | 4.2   |
| 0.9        | 0.8         | 0.13 | 4.2   |
| 0.8        | 0.7         | 0.13 | 4.1   |
| 0.7        | 0.7         | 0.12 | 4.1   |
| 0.6        | 0.6         | 0.11 | 4.1   |
| 0.5        | 0.5         | 0.10 | 4.1   |
| 0.4        | 0.5         | 0.9  | 4.1   |
| 0.3        | 0.4         | 0.8  | 4.1   |

## Project Status

All Spine features are implemented. If you notice something is broken, please submit an issue. The Bevy API needs a lot of work and feedback is welcome.

## License

This code is licensed under dual MIT / Apache-2.0 but with no attribution necessary. All contributions must agree to this licensing.

Please note that this project uses the Spine Runtime and to use it you must follow the [Spine Runtimes License Agreement](https://github.com/EsotericSoftware/spine-runtimes/blob/4.1/LICENSE).
