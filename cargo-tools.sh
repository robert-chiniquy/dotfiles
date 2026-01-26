#!/bin/bash
# Cargo tools to install to ~/bin/
# Run: ./cargo-tools.sh

set -e

mkdir -p ~/bin

TOOLS=(
  fclones      # duplicate file finder
)

for tool in "${TOOLS[@]}"; do
  # Skip comments
  [[ "$tool" == \#* ]] && continue
  echo "Installing $tool..."
  cargo install "$tool" --root ~/
done

echo "Done. Tools installed to ~/bin/"
