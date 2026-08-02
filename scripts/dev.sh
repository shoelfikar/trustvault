#!/usr/bin/env bash
# Launch `tauri dev` with the snap environment stripped out.
#
# Why this exists: when the editor is installed as a snap (VS Code, for one), its integrated
# terminal exports SNAP_LIBRARY_PATH, LOCPATH, GTK_PATH, GIO_MODULE_DIR and friends pointing
# into /snap/core*/. A natively-built binary launched from that terminal then loads the snap's
# libc and dies before main():
#
#   symbol lookup error: /snap/core20/current/lib/x86_64-linux-gnu/libpthread.so.0:
#   undefined symbol: __libc_pthread_init, version GLIBC_PRIVATE
#
# The binary is fine; the environment is not. Running `npm run tauri dev` from a terminal
# outside the snap works without this script.
set -euo pipefail

if [[ -z "${SNAP:-}" ]]; then
  echo "Not inside a snap environment — running tauri dev directly." >&2
  exec npm run tauri dev -- "$@"
fi

echo "Snap environment detected (${SNAP_NAME:-unknown}); stripping it for the child process." >&2

unset SNAP SNAP_ARCH SNAP_COMMON SNAP_CONTEXT SNAP_COOKIE SNAP_DATA SNAP_EUID \
  SNAP_INSTANCE_NAME SNAP_LAUNCHER_ARCH_TRIPLET SNAP_LIBRARY_PATH SNAP_NAME \
  SNAP_REAL_HOME SNAP_REVISION SNAP_UID SNAP_USER_COMMON SNAP_USER_DATA SNAP_VERSION \
  LD_LIBRARY_PATH LOCPATH GTK_PATH GTK_EXE_PREFIX GTK_IM_MODULE_FILE GTK_MODULES \
  GIO_MODULE_DIR GSETTINGS_SCHEMA_DIR GDK_PIXBUF_MODULEDIR GDK_PIXBUF_MODULE_FILE

# Drop the snap's bin directories from PATH so node and cargo resolve to the host copies.
PATH="$(printf '%s' "$PATH" | tr ':' '\n' | grep -v '^/snap/' | paste -sd:)"
export PATH

exec npm run tauri dev -- "$@"
