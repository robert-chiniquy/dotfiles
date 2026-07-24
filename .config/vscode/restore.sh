#!/bin/sh
# Reinstall VS Code extensions + link settings on a new machine.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
dest="$HOME/Library/Application Support/Code/User"
mkdir -p "$dest"
[ -e "$dest/settings.json" ] || ln -s "$here/settings.json" "$dest/settings.json"
while IFS= read -r ext; do [ -n "$ext" ] && code --install-extension "$ext" || true; done < "$here/extensions.txt"
echo "VS Code: extensions installed, settings linked."
