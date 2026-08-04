#!/bin/sh
set -eu

if [ "$(/usr/bin/uname -s)" != "Darwin" ]; then
  echo "scorecard token refresh requires macOS launchd" >&2
  exit 69
fi

TOKEN_INSTALL_SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
TOKEN_INSTALL_ROOT=$(dirname -- "$TOKEN_INSTALL_SCRIPT_DIR")
TOKEN_INSTALL_LABEL=com.rch.scorecard-token-refresh
TOKEN_INSTALL_SOURCE="$TOKEN_INSTALL_ROOT/LaunchAgents/$TOKEN_INSTALL_LABEL.plist"
TOKEN_INSTALL_TARGET="$HOME/Library/LaunchAgents/$TOKEN_INSTALL_LABEL.plist"
TOKEN_INSTALL_DOMAIN="gui/$(/usr/bin/id -u)"
TOKEN_INSTALL_SERVICE="$TOKEN_INSTALL_DOMAIN/$TOKEN_INSTALL_LABEL"

/bin/mkdir -p "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"

if [ -e "$TOKEN_INSTALL_TARGET" ] && [ ! -L "$TOKEN_INSTALL_TARGET" ]; then
  echo "$TOKEN_INSTALL_TARGET exists and is not a symlink" >&2
  exit 73
fi

"$TOKEN_INSTALL_SCRIPT_DIR/update-weekly-token-scorecard.sh"
/bin/ln -sfn "$TOKEN_INSTALL_SOURCE" "$TOKEN_INSTALL_TARGET"

/bin/launchctl bootout "$TOKEN_INSTALL_SERVICE" 2>/dev/null || true
/bin/launchctl bootstrap "$TOKEN_INSTALL_DOMAIN" "$TOKEN_INSTALL_TARGET"
/bin/launchctl enable "$TOKEN_INSTALL_SERVICE"
/bin/launchctl kickstart -k "$TOKEN_INSTALL_SERVICE"

echo "installed $TOKEN_INSTALL_SERVICE"
echo "card: $HOME/.config/scorecard/weekly-agent-tokens.md"
echo "report: $HOME/.local/share/scorecard/weekly-agent-tokens.json"
