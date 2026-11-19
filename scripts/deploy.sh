#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ -n "${RYUJINX_DIR:-}" ]]; then
    ryujinx_dir="$RYUJINX_DIR"
else
    if ! command -v cmd.exe >/dev/null || ! command -v wslpath >/dev/null; then
        echo "Ryujinx introuvable : définis RYUJINX_DIR." >&2
        exit 1
    fi
    appdata="$(cmd.exe /c 'echo %APPDATA%' 2>/dev/null | tr -d '\r')"
    if [[ -z "$appdata" || "$appdata" == *'%APPDATA%'* ]]; then
        echo "APPDATA introuvable : définis RYUJINX_DIR." >&2
        exit 1
    fi
    ryujinx_dir="$(wslpath -u "$appdata")/Ryujinx"
fi

dest_dir="$ryujinx_dir/sdcard/engage/mods/Class_Skill_Icon_Sync"
dest="$dest_dir/libclass_skill_icon_sync.nro"
source_nro="target/aarch64-skyline-switch/release/libclass_skill_icon_sync.nro"

cargo skyline build --release
if [[ ! -f "$source_nro" ]]; then
    echo "NRO non créé : $source_nro" >&2
    exit 1
fi

mkdir -p -- "$dest_dir"
if [[ -f "$dest" && ! -e "$dest.bak" ]]; then
    cp -p -- "$dest" "$dest.bak"
fi
cp -- "$source_nro" "$dest"
echo "NRO installé : $dest"
