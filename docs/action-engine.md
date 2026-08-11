# Action engine

Phase 4 adds structured actions to the Rust search pipeline. Actions are stored in a bounded core catalog and executed by opaque IDs. The frontend cannot submit an executable, script or clipboard payload.

## Calculator

The internal parser supports decimal numbers, parentheses, unary signs, `+`, `-`, `*`, `/`, `%`, `^` and `sqrt`. Exponentiation is right-associative and precedes unary minus. Division by zero, unknown functions, malformed input and non-finite results are rejected. JavaScript `eval` is never used.

## Offline conversions

Supported pairs currently include kilometers/miles, meters/feet, Celsius/Fahrenheit, GB/MB and kilograms/pounds. Binary data conversion uses 1024. Currency is deliberately excluded until an external provider, freshness policy and cache contract are selected.

## System actions

- `lock` executes immediately.
- `sleep`, `restart` and `shutdown` require two consecutive Enter presses on the same selected result. Escape cancels.
- Windows uses system executables with separated arguments. `SetSuspendState` behavior may be affected by hibernation policy.
- macOS uses System Events through `osascript` and may request Automation permission.
- Linux uses `loginctl lock-session` and `systemctl` for power actions; desktop policy/Polkit can deny them.

The core rechecks confirmation. UI confirmation alone is not a security boundary.

## Clipboard

Calculation and conversion results use the official Tauri clipboard manager from Rust. Only write access is used. No clipboard history or read permission is introduced in this phase.
