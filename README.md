# Work in progress
A virtual input device manager - written in Rust.

![table_ui](https://github.com/ArrowMaxGithub/Rust-vJoy-Manager/assets/60489413/84c52a0e-d301-455c-bfbf-da1814420b1b)

## Overview
**Reroute input:**
- Combine input from multiple gamepads, joysticks, throttles etc. to one virtual joystick.

**Transform input:**
- Create analog axes from buttons. 
- Apply button- or axis-trim to existing axes.
- Create tempo or toggle buttons from momentary buttons.

**Shift-modes:**
- Assign multiple output rebinds to one input via shift-modes.

## Requirements
Windows 10/11 64 bit only for now. Linux support is blocked by a missing vJoy alternative.

[vJoy driver](https://github.com/njz3/vJoy/) version 2.2.1.1 needs to be installed.

## State
Rebind maps can be created one rebind at a time with the existing UI.
Rebinds can be edited and saved/loaded to/from a custom location.
Input is properly transformed and piped.

The existing set of rebind types is enough to setup a proper flight sim configuration, but the setup takes a while.

## Todo
- Easier rebind setup with quick-configurations ('setup wizard').

- More rebind variants:
    - Split hat into 4/8 buttons.
    - Combine buttons to hats.
    - More axes merge options.
    - Split axis into +/- component.

- Documentation/Guide for the available rebinds.

- In-application console for logging.

## Build
Either call one of the launch batchfiles from the workspace root or build and copy to output manually:

Debug build:
- `cargo build`
- Copy `assets/` to `target/debug`
- Copy `SDL2.dll` to `target/debug`

Profiling build:
- `cargo build  --release  --features "profile"`
- Copy `assets/` to `target/release`
- Copy `SDL2.dll` to `target/release`
- Start [Tracy](https://github.com/nagisa/rust_tracy_client) client and connect running instance to collect traces.

Dist build:
- `cargo build --profile dist`
- Copy `assets/` to `target/dist`
- Copy `SDL2.dll` to `target/dist`

## Technical infos
Input is read through [SDL2](https://github.com/Rust-SDL2/rust-sdl2) as generic joystick input.

Output is piped to [vJoy](https://github.com/njz3/vJoy/) through a [Rust wrapper library](https://github.com/ArrowMaxGithub/vjoy).

The GUI of choice: [egui](https://github.com/emilk/egui).

Which is rendered via [Vulkan](https://www.vulkan.org/) through a custom wrapper: [VKU](https://github.com/ArrowMaxGithub/vku).

Rebind maps are de-/serialized to [TOML](https://github.com/toml-rs/toml) through the [Serde](https://github.com/serde-rs/serde) framework.

Profiling is supported through [Tracy](https://github.com/nagisa/rust_tracy_client).
