## Basic Usage
1. Enable any required virtual devices via vJoy.
2. Start rust-vjoy-manager.exe and prepare your config. Any changes are auto-live and may be visualized by the input viewer.
3. Save config via System -> Save config.
4. The most recently saved config will be auto-loaded on next start.

## Rebind types
- Logical rebinds: don't modify or pipe any input, but prepare information/state which is used by reroute and virtual rebinds.
- Reroute rebinds: pipe input from physical devices to virtual devices and may transform input (e.g. combine two buttons to one axis or apply axis offsets).
- Virtual rebinds: act on the state of virtual devices exclusively.

## Execution order
Rebinds are processed in the order of logical -> reroute -> virtual. Within one rebind type the order is top-to-bottom.

## Shift modes
Shift modes can enable/disable rebinds and act as a bitmask.

`0b00000001` enables any rebind that requires the first bit to be set.

`0b00000000` disables any rebind that requires the first bit to be set.

`Active mode` is the required bitmask for a rebind to be considered active.

The current shift mode is found just below the `Virtual devices` label. Default mode: `0b00000001`.

## Adding/Removing rebinds
Rebinds can be added via `Add logical/reroute/virtual` buttons at the top of the rebind list.

To remove/reorder a rebind, use the buttons next to the rebind.

## Input viewer
Any rebinds in the loaded configuration are live. You can visualize the input for mutliple devices at the time by clicking the physical/virtual device in the devices list.

Tesselation/rendering of the input plots is quite CPU-intensive. You can minimize RVM to save resources and only process your rebinds without any rendering.

## Logs/Errors
The terminal alongside the application will log information and errors - proper file logs are in the works.