#!/bin/bash
set -e

# === Gum helpers (fall back to echo if not installed) ===
if command -v gum &>/dev/null; then
  header() { gum style --foreground="#ff00f8" --bold --border="rounded" --border-foreground="#5cecff" --padding="0 2" "$1"; }
  info() { gum style --foreground="#5cecff" "  $1"; }
  success() { gum style --foreground="#5cecff" --bold "  [OK] $1"; }
  warn() { gum style --foreground="#fbb725" "  [!] $1"; }
  spin() { gum spin --spinner="dot" --spinner.foreground="#ff00f8" --title="$1" -- "${@:2}"; }
else
  header() { echo -e "\n=== $1 ===\n"; }
  info() { echo "  $1"; }
  success() { echo "  [OK] $1"; }
  warn() { echo "  [!] $1"; }
  spin() { echo "$1..."; "${@:2}"; }
fi

UNLINK_DIR_FLAG=''

# Ubuntu unlink doesn't take -d
[ -e /etc/os-release ] && unset UNLINK_DIR_FLAG

is_macos() { [[ "$OSTYPE" == darwin* ]]; }

# Real (non-symlink) directories in the way of a link are moved here, never
# deleted. The directory is only created if something actually needs saving.
BACKUP_DIR="$HOME/.dotfiles-backup/$(date +%Y%m%d-%H%M%S)"
backup_target() {
  local dest
  mkdir -p "$BACKUP_DIR"
  dest="$BACKUP_DIR/$(basename "$1")"
  while [ -e "$dest" ]; do dest="$dest.dup"; done
  mv "$1" "$dest" && warn "Backed up $1 -> $dest"
}

[ -e .git ] || { echo ":/  Run from dotfiles repo root"; exit 1; }

mkdir -p ~/.config

# === Homebrew packages ===
header "Homebrew Packages"
if command -v brew &>/dev/null && [ -f Brewfile ]; then
  spin "Installing packages" brew bundle --no-lock 2>/dev/null || warn "Some packages failed"
  success "Homebrew done"
else
  warn "Homebrew not found, skipping"
fi

# === Unlink existing symlinks ===
header "Unlinking Old Symlinks"
unlink_if_exists() {
  if [ -h "$1" ]; then
    unlink $UNLINK_DIR_FLAG "$1" && info "Unlinked $1"
  elif [ -d "$1" ]; then
    backup_target "$1"
  fi
}

unlink_if_exists ~/.vim
unlink_if_exists ~/.vimrc
unlink_if_exists ~/.bash_login
unlink_if_exists ~/.inputrc
unlink_if_exists ~/.zprofile
unlink_if_exists ~/.zshenv
unlink_if_exists ~/.zshrc
unlink_if_exists ~/.claude
unlink_if_exists ~/.grok
unlink_if_exists ~/.gitconfig
unlink_if_exists ~/.gitignore_global
unlink_if_exists ~/.ripgreprc
unlink_if_exists ~/.tmux.conf
unlink_if_exists ~/.config/starship.toml
unlink_if_exists ~/.config/bat
unlink_if_exists ~/.config/glow
unlink_if_exists ~/.config/ghostty
unlink_if_exists ~/.config/nvim
unlink_if_exists ~/.config/yazi
unlink_if_exists ~/.config/atuin
unlink_if_exists ~/bin
# Nushell config location differs per OS
if is_macos; then
  unlink_if_exists ~/Library/Application\ Support/nushell
else
  unlink_if_exists ~/.config/nushell
fi
# Window management + Übersicht widgets (macOS only)
if is_macos; then
  unlink_if_exists ~/.yabairc
  unlink_if_exists ~/.skhdrc
  unlink_if_exists ~/.config/sketchybar
  unlink_if_exists ~/.config/borders
  unlink_if_exists ~/.config/yabai
  unlink_if_exists ~/.hammerspoon
  unlink_if_exists ~/Library/Application\ Support/Übersicht/widgets
fi
success "Cleanup done"

# === Create symlinks ===
header "Creating Symlinks"
link_if_missing() {
  if [ ! -e "$2" ]; then
    ln -s "$(pwd)/$1" "$2" && info "$1 -> $2"
  fi
}

link_if_missing .vim ~/.vim
link_if_missing .vimrc ~/.vimrc
link_if_missing .bash_login ~/.bash_login
link_if_missing .inputrc ~/.inputrc
link_if_missing .zprofile ~/.zprofile
link_if_missing .zshenv ~/.zshenv
link_if_missing .zshrc ~/.zshrc
link_if_missing .claude ~/.claude
link_if_missing .grok ~/.grok
link_if_missing .gitconfig ~/.gitconfig
link_if_missing .gitignore_global ~/.gitignore_global
link_if_missing .ripgreprc ~/.ripgreprc
link_if_missing .tmux.conf ~/.tmux.conf
link_if_missing starship.toml ~/.config/starship.toml
link_if_missing .config/bat ~/.config/bat
link_if_missing .config/glow ~/.config/glow
link_if_missing .config/ghostty ~/.config/ghostty
link_if_missing .config/nvim ~/.config/nvim
link_if_missing .config/yazi ~/.config/yazi
link_if_missing .config/atuin ~/.config/atuin
link_if_missing .config/erdtree ~/.config/erdtree
link_if_missing bin ~/bin
# Nushell config location differs per OS
if is_macos; then
  mkdir -p ~/Library/Application\ Support
  link_if_missing .config/nushell ~/Library/Application\ Support/nushell
else
  link_if_missing .config/nushell ~/.config/nushell
fi
# Window management + Übersicht widgets (macOS only)
if is_macos; then
  link_if_missing .yabairc ~/.yabairc
  link_if_missing .skhdrc ~/.skhdrc
  link_if_missing .config/sketchybar ~/.config/sketchybar
  link_if_missing .config/borders ~/.config/borders
  link_if_missing .config/yabai ~/.config/yabai
  link_if_missing .hammerspoon ~/.hammerspoon
  mkdir -p ~/Library/Application\ Support/Übersicht
  link_if_missing ubersicht-widgets ~/Library/Application\ Support/Übersicht/widgets
fi
success "Symlinks done"

# === Shared agent skills ===
header "Shared Agent Skills"
if [ -x scripts/install-shared-agent-skills.sh ]; then
  spin "Linking canonical skill tree" scripts/install-shared-agent-skills.sh
  success "Shared agent skills installed"
fi

# === Codex guidance ===
header "Codex Guidance"
if [ -x scripts/install-codex-guidance.sh ]; then
  spin "Installing global guidance" scripts/install-codex-guidance.sh
  success "Codex guidance installed"
fi

# === Post-install ===
header "Post-Install"
# Initialize only submodules declared in .gitmodules (stray gitlinks in the
# index would otherwise abort the whole install). Not run through spin: gum
# spin executes binaries, not shell loops.
info "Updating submodules"
git config --file .gitmodules --get-regexp '\.path$' | while read -r _ p; do
  git submodule update --init -- "$p"
done

if command -v bat &>/dev/null; then
  spin "Rebuilding bat cache" bat cache --build
  success "Bat cache rebuilt"
fi

# === Done ===
echo ""
if [ -d "$BACKUP_DIR" ]; then
  warn "Pre-existing directories were moved to $BACKUP_DIR"
fi
if command -v gum &>/dev/null; then
  gum style \
    --foreground="#5cecff" \
    --border="double" \
    --border-foreground="#ff00f8" \
    --padding="1 3" \
    --bold \
    "DONE" "" "Restart shell:" "  zsh: source ~/.zshrc" "  nu:  nu"
else
  echo "Done! Restart your shell:"
  echo "  zsh: source ~/.zshrc"
  echo "  nu:  nu"
fi
