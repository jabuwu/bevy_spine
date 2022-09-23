# bevy_spine

A Bevy Plugin for Spine, utilizing [rusty_spine](https://github.com/jabuwu/rusty_spine). WASM compatible!

## Project Status

All Spine features (IK, Paths, Clipping, etc) work. Some models might depend on backface culling being disabled or blend modes. but the default 2D Bevy renderer does not support these (as far as I know). Because of this, a custom renderer may need to be used.

The rusty_spine API still needs some work. Attachments and timelines in particular cannot be manipulated without relying on the C API directly.

## License

This code is licensed under dual MIT / Apache-2.0 but with no attribution necessary. All contributions must agree to this licensing.

Please note that this project uses the Spine Runtime and to use it you must follow the [Spine Runtimes License Agreement](https://github.com/EsotericSoftware/spine-runtimes/blob/4.1/LICENSE).
