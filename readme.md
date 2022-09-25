# bevy_spine

A Bevy Plugin for Spine, utilizing [rusty_spine](https://github.com/jabuwu/rusty_spine). WASM compatible!

```
[dependencies]
bevy = "0.8"
bevy_spine = "0.1.0"
```

## Versions

| bevy_spine  | rusty_spine | bevy | spine |
| ----------- | ----------- | ---- | ----- |
| 0.1.0       | 0.2.0       | 0.8  | 4.1   |

## Project Status

All Spine features (IK, Paths, Clipping, etc) work. Some models might depend on backface culling being disabled or blend modes. but the default 2D Bevy renderer does not support these (as far as I know). Because of this, a custom renderer may need to be used.

The Bevy API needs a lot of work and feedback is welcome.

## License

This code is licensed under dual MIT / Apache-2.0 but with no attribution necessary. All contributions must agree to this licensing.

Please note that this project uses the Spine Runtime and to use it you must follow the [Spine Runtimes License Agreement](https://github.com/EsotericSoftware/spine-runtimes/blob/4.1/LICENSE).
