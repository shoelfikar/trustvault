#!/usr/bin/env bash
# Launch `tauri dev` in an environment the editor's snap has not touched.
#
# Why this exists: when the editor is installed as a snap (VS Code, for one), its integrated
# terminal exports LD_LIBRARY_PATH, LOCPATH, GTK_PATH, GIO_MODULE_DIR, GDK_PIXBUF_MODULE_FILE,
# XDG_DATA_DIRS and friends pointing into /snap/. A natively-built binary launched from that
# terminal then loads the snap's glibc 2.31 alongside the host's, and dies before main():
#
#   symbol lookup error: /snap/core20/current/lib/x86_64-linux-gnu/libpthread.so.0:
#   undefined symbol: __libc_pthread_init, version GLIBC_PRIVATE
#
# The same fault also arrives later and indirectly: GDK_PIXBUF_MODULE_FILE and the GTK
# immodules cache list loader .so files under /snap/code/*/, so GTK dlopens snap objects
# built against that older glibc even when the main binary linked cleanly.
#
# The binary is fine; the environment is not. Running `npm run tauri dev` from a terminal
# outside the snap works without this script.
#
# The child environment is built from an allowlist rather than by unsetting known-bad names.
# A blacklist has to be updated every time a snap revision exports something new, and it
# already missed XDG_DATA_DIRS and XDG_DATA_HOME.
set -euo pipefail

if [[ -z "${SNAP:-}" ]]; then
  echo "Not inside a snap environment — running tauri dev directly." >&2
  exec npm run tauri dev -- "$@"
fi

echo "Snap environment detected (${SNAP_NAME:-unknown}); rebuilding a clean environment." >&2

# Some snaps remap HOME into ~/snap/<name>/<rev>; cargo and npm must see the real one.
home="${SNAP_REAL_HOME:-$HOME}"

# Drop the snap's bin directories so node, npm and cargo resolve to the host copies.
clean_path="$(printf '%s' "$PATH" | tr ':' '\n' | grep -v '^/snap/' | paste -sd:)"

# Session and toolchain variables that carry no snap paths and that the app or the build
# genuinely needs. Anything not named here does not reach the child.
keep=()
for var in USER LOGNAME SHELL TERM COLORTERM LANG LC_ALL LC_CTYPE \
  XDG_RUNTIME_DIR XDG_SESSION_TYPE XDG_CURRENT_DESKTOP \
  DISPLAY WAYLAND_DISPLAY XAUTHORITY DBUS_SESSION_BUS_ADDRESS \
  CARGO_HOME RUSTUP_HOME RUST_BACKTRACE RUST_LOG SSH_AUTH_SOCK; do
  if [[ -n "${!var:-}" ]]; then
    keep+=("$var=${!var}")
  fi
done

exec env -i \
  HOME="$home" \
  PATH="$clean_path" \
  XDG_DATA_DIRS=/usr/local/share:/usr/share \
  XDG_CONFIG_DIRS=/etc/xdg \
  "${keep[@]}" \
  npm run tauri dev -- "$@"
