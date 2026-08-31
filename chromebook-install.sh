#!/bin/bash
set -euo pipefail

# Chromebook (Crostini) subset of install.sh: editors and terminal comfort
# only. Deliberately NOT installed here: agent config (.claude, .grok,
# ~/.agents skill tree, codex guidance), shell rc files, Homebrew, and macOS
# window management. Coding agents are not permitted on Chromebooks, and the
# container's stock bash environment is left alone. Git aliases are MERGED
# into the existing ~/.gitconfig so the machine's identity and credential
# setup survive.

[ -e .git ] || { echo "Run from dotfiles repo root" >&2; exit 1; }
if [ "$(uname -s)" = "Darwin" ]; then
  echo "This is the Chromebook/Linux subset; use ./install.sh on macOS" >&2
  exit 1
fi

info() { echo "  $1"; }
warn() { echo "  [!] $1"; }

# Anything real in the way is moved here, never deleted.
BACKUP_DIR="$HOME/.dotfiles-backup/$(date +%Y%m%d-%H%M%S)"

backup_then_link() {
  local src dst dest
  src="$(pwd)/$1"
  dst="$2"
  if [ -h "$dst" ]; then
    unlink "$dst"
  elif [ -e "$dst" ]; then
    mkdir -p "$BACKUP_DIR"
    dest="$BACKUP_DIR/$(basename "$dst")"
    while [ -e "$dest" ]; do dest="$dest.dup"; done
    mv "$dst" "$dest"
    warn "Backed up $dst -> $dest"
  fi
  ln -s "$src" "$dst" && info "$1 -> $dst"
}

backup_then_link .vim ~/.vim
backup_then_link .vimrc ~/.vimrc
backup_then_link .tmux.conf ~/.tmux.conf
backup_then_link .inputrc ~/.inputrc
backup_then_link .ripgreprc ~/.ripgreprc

# Merge [alias] entries from the repo .gitconfig into the global config.
while IFS= read -r name; do
  git config --global "$name" "$(git config --file .gitconfig --get "$name")"
done < <(git config --file .gitconfig --name-only --get-regexp '^alias\.')
info "git aliases merged into ~/.gitconfig"

# Initialize only submodules declared in .gitmodules (stray gitlinks in the
# index would otherwise abort the whole install).
git config --file .gitmodules --get-regexp '\.path$' | while read -r _ p; do
  git submodule update --init -- "$p"
done

echo "Done."
if [ -d "$BACKUP_DIR" ]; then
  warn "Pre-existing files were moved to $BACKUP_DIR"
fi
