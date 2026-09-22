#!/bin/sh
set -eu
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PACKAGE="$SCRIPT_DIR/kwin/cooldeadpipewire-active-window"
kpackagetool6 --type=KWin/Script -i "$PACKAGE"
kwriteconfig6 --file kwinrc --group Plugins --key cooldeadpipewire-active-windowEnabled true
qdbus6 org.kde.KWin /KWin reconfigure
echo "Installed and enabled CooldeadPipeWire Active Window Bridge."
