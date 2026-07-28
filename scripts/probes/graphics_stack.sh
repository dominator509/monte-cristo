#!/usr/bin/env sh
# Read-only probe: the host can link mc_shell against its windowing and audio stack.
# Inspects only; installs nothing and opens no window.
set -eu
stack="${MC_GRAPHICS_STACK:-}"
[ -n "$stack" ] || { echo "probe graphics_stack: MC_GRAPHICS_STACK not set" >&2; exit 1; }
case "$stack" in
  x11)
    for h in /usr/include/X11/Xlib.h /usr/include/GL/gl.h; do
      [ -f "$h" ] || { echo "probe graphics_stack: missing header $h (install X11 and GL dev packages)" >&2; exit 1; }
    done
    [ -f /usr/include/alsa/asoundlib.h ] || { echo "probe graphics_stack: missing alsa dev headers" >&2; exit 1; }
    ;;
  wayland)
    [ -f /usr/include/wayland-client.h ] || { echo "probe graphics_stack: missing wayland-client.h" >&2; exit 1; }
    [ -f /usr/include/alsa/asoundlib.h ] || { echo "probe graphics_stack: missing alsa dev headers" >&2; exit 1; }
    ;;
  macos)
    command -v xcrun >/dev/null 2>&1 || { echo "probe graphics_stack: xcrun not found (install Xcode command line tools)" >&2; exit 1; }
    ;;
  mingw)
    command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1 || { echo "probe graphics_stack: mingw-w64 linker not found" >&2; exit 1; }
    ;;
  *)
    echo "probe graphics_stack: MC_GRAPHICS_STACK must be one of x11, wayland, macos, mingw" >&2; exit 1 ;;
esac
echo "probe graphics_stack: ok"
