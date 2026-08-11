# Platform support

## Windows

Application discovery inspects real Start Menu locations and packaged-app metadata through a Windows adapter. WebView2 is the renderer. The global shortcut is configurable from a bounded core-owned list; conflicts still require version-specific tests.

## macOS

Discovery inspects application bundles through macOS APIs. `Command+Space` normally conflicts with Spotlight; users can select a different persisted combination and failed changes roll back. Accessibility and automation permissions are feature-specific.

## Linux

Discovery parses XDG `.desktop` entries. Window positioning, focus and global shortcuts vary between X11, Wayland and compositors. Shortcut choices are configurable, but compositor support still needs manual acceptance. WebKitGTK availability is a deployment constraint; the current surface is opaque.

The project does not claim parity until CI builds and manual OS-specific acceptance tests exist.

## Power actions

Windows power actions use built-in executables; sleep behavior depends on system hibernation settings. macOS actions use System Events and can require Automation permission. Linux uses logind/systemd interfaces and may be denied by Polkit or unavailable on non-systemd distributions. Destructive actions always require explicit in-app confirmation.
