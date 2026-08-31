# worms

A Rust workspace for native larvae worms.

- [`vertical-spacing`](crates/vertical-spacing) formats plain Luau with deterministic vertical-spacing conventions.

## Development

```text
mise install
mise run check
mise run build
```

## Releases

Every worm shares the version under `[workspace.package]`. A matching `v*` tag builds native archives for Linux, macOS, and Windows, tests the Linux archives over the native protocol, and creates a GitHub release containing every worm.

Release assets follow larvae's platform naming convention:

```text
vertical-spacing-worm-x86_64-linux.zip
vertical-spacing-worm-aarch64-linux.zip
vertical-spacing-worm-x86_64-macos.zip
vertical-spacing-worm-aarch64-macos.zip
vertical-spacing-worm-x86_64-windows.zip
```

A project installs one worm from the shared repository by name:

```text
larvae worm add owner/worms@0.1.0 --name vertical-spacing
larvae worm install
```
