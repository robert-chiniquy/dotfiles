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
    rm -rf "$1" && info "Removed $1"
  fi
}

unlink_if_exists ~/.vim
unlink_if_exists ~/.vimrc
unlink_if_exists ~/.bash_login
unlink_if_exists ~/.inputrc
unlink_if_exists ~/.zprofile
unlink_if_exists ~/.zshrc
unlink_if_exists ~/.claude
unlink_if_exists ~/.gitconfig
unlink_if_exists ~/.ripgreprc
unlink_if_exists ~/.tmux.conf
unlink_if_exists ~/.config/starship.toml
unlink_if_exists ~/.config/bat
unlink_if_exists ~/.config/glow
unlink_if_exists ~/.config/ghostty
unlink_if_exists ~/.config/nvim
unlink_if_exists ~/.config/yazi
unlink_if_exists ~/bin
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
link_if_missing .zshrc ~/.zshrc
link_if_missing .claude ~/.claude
link_if_missing .gitconfig ~/.gitconfig
link_if_missing .ripgreprc ~/.ripgreprc
link_if_missing .tmux.conf ~/.tmux.conf
link_if_missing starship.toml ~/.config/starship.toml
link_if_missing .config/bat ~/.config/bat
link_if_missing .config/glow ~/.config/glow
link_if_missing .config/ghostty ~/.config/ghostty
link_if_missing .config/nvim ~/.config/nvim
link_if_missing .config/yazi ~/.config/yazi
link_if_missing bin ~/bin
success "Symlinks done"

# === Post-install ===
header "Post-Install"
spin "Updating submodules" git submodule update --init

if command -v bat &>/dev/null; then
  spin "Rebuilding bat cache" bat cache --build
  success "Bat cache rebuilt"
fi

# === Done ===
echo ""
if command -v gum &>/dev/null; then
  gum style \
    --foreground="#5cecff" \
    --border="double" \
    --border-foreground="#ff00f8" \
    --padding="1 3" \
    --bold \
    "DONE" "" "Restart shell or: source ~/.zshrc"
else
  echo "Done! Restart your shell or run: source ~/.zshrc"
fi
