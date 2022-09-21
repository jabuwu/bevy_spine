# bevy_spine

A Bevy Plugin for Spine, utilizing [rusty_spine](https://github.com/jabuwu/rusty_spine). WASM compatible!

## Project Status

All Spine features (IK, Paths, Clipping, etc) work. Some models might depend on backface culling being disabled or blend modes. but the default 2D Bevy renderer does not support these (as far as I know). Because of this, a custom renderer may need to be used.

The rusty_spine API still needs a lot of work. Simple things like Bone manipulation works fine, but working directly with attachments and track entries is not possible without falling back on the C API.

The skeleton should probably sync with the Bevy entity hierarchy, that way it's possible to interact directly with Bevy APIs rather than having to work entirely through the Skeleton API.

## License

This code is licensed under dual MIT / Apache-2.0 but with no attribution necessary. All contributions must agree to this licensing.

Please note that this project uses the Spine Runtime and to use it you must follow the [Spine Runtimes License Agreement](https://github.com/EsotericSoftware/spine-runtimes/blob/4.1/LICENSE).