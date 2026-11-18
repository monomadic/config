# CoderBar vendored build

Source: https://github.com/monomadic/CoderBar
Upstream: https://github.com/helloyulife/CoderBar
Pinned commit: `f1fd52e` (0.5.5, macOS arm64).

`vendor/bin/coder-bar`, `coder-bar-hook`, and `coder-bar-ctl` are the
executables from that build. This directory contains its bundle metadata
and icons. No settings, session data, or credentials are included.

Install with `bash scripts/install/install-coderbar.sh`, then launch
`open /Applications/CoderBar.app`. The installer reconstructs and ad-hoc
signs the app; it does not change login or notification preferences.
The usual dotfiles deployment exposes the wrappers in `bin/` on PATH.

To refresh, build and package the source checkout, copy the three executables
from `dist/CoderBar.app/Contents/MacOS/` into `vendor/bin/`, and copy its
Info.plist and Resources into this directory. Update the pinned commit,
run the installer, and restart the app.

The bundle icon is the peeled Space Chocolate candy-wrapper concept (`14-space-chocolate-coderbar`). It is loaded from the static AppIcon.icns resource.

The installer uses `set-app-icon.js` with built-in osascript to apply a Finder custom icon after signing, preserving the wrapper silhouette on macOS Tahoe. Strict signing verification runs before adding Finder metadata; ordinary signature verification runs afterward. No runtime icon override is used.
