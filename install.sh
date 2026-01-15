#! /bin/bash

set -e -x

UNLINK_DIR_FLAG=''

# ubuntu unlink doesn't take -d
[ -e /etc/os-release ] && unset UNLINK_DIR_FLAG

[ `which locale-gen` ] && locale-gen en_US.UTF-8

[ -e .git ] || ( echo ":/" ; exit 1 )

mkdir -p ~/.config

# === Homebrew packages (macOS only) ===
if command -v brew &>/dev/null && [ -f Brewfile ]; then
  echo "Installing Homebrew packages..."
  brew bundle --no-lock || echo "Some packages failed to install"
fi

# === Unlink existing symlinks ===
[ -h ~/.vim ] && unlink $UNLINK_DIR_FLAG ~/.vim
[ -h ~/.vimrc ] && unlink ~/.vimrc
[ -h ~/.bash_login ] && unlink ~/.bash_login
[ -h ~/.inputrc ] && unlink ~/.inputrc
[ -h ~/.zprofile ] && unlink ~/.zprofile
[ -h ~/.zshrc ] && unlink ~/.zshrc
[ -h ~/.claude ] && unlink $UNLINK_DIR_FLAG ~/.claude
[ -h ~/.gitconfig ] && unlink ~/.gitconfig
[ -h ~/.ripgreprc ] && unlink ~/.ripgreprc
[ -h ~/.config/starship.toml ] && unlink ~/.config/starship.toml
[ -h ~/.config/bat ] && unlink $UNLINK_DIR_FLAG ~/.config/bat
[ -h ~/.config/glow ] && unlink $UNLINK_DIR_FLAG ~/.config/glow
[ -h ~/.config/ghostty ] && unlink $UNLINK_DIR_FLAG ~/.config/ghostty
[ -h ~/bin ] && unlink $UNLINK_DIR_FLAG ~/bin

# === Create symlinks ===
[ ! -e ~/.vim ] && ln -s `pwd`/.vim ~/.vim
[ ! -e ~/.vimrc ] && ln -s `pwd`/.vimrc ~/.vimrc
[ ! -e ~/.bash_login ] && ln -s `pwd`/.bash_login ~/.bash_login
[ ! -e ~/.inputrc ] && ln -s `pwd`/.inputrc ~/.inputrc
[ ! -e ~/.zprofile ] && ln -s `pwd`/.zprofile ~/.zprofile
[ ! -e ~/.zshrc ] && ln -s `pwd`/.zshrc ~/.zshrc
[ ! -e ~/.claude ] && ln -s `pwd`/.claude ~/.claude
[ ! -e ~/.gitconfig ] && ln -s `pwd`/.gitconfig ~/.gitconfig
[ ! -e ~/.ripgreprc ] && ln -s `pwd`/.ripgreprc ~/.ripgreprc
[ ! -e ~/.config/starship.toml ] && ln -s `pwd`/starship.toml ~/.config/starship.toml
[ ! -e ~/.config/bat ] && ln -s `pwd`/.config/bat ~/.config/bat
[ ! -e ~/.config/glow ] && ln -s `pwd`/.config/glow ~/.config/glow
[ ! -e ~/.config/ghostty ] && ln -s `pwd`/.config/ghostty ~/.config/ghostty
[ ! -e ~/bin ] && ln -s `pwd`/bin ~/bin

# === Post-install ===
git submodule update --init

# Rebuild bat cache if bat is installed
if command -v bat &>/dev/null; then
  echo "Rebuilding bat theme cache..."
  bat cache --build
fi

echo "Done! Restart your shell or run: source ~/.zshrc"
