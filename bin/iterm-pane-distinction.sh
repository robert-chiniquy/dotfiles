#!/bin/bash
# Make active iTerm2 pane/tab visually distinct from inactive ones.
# Vaporwave-friendly: inactive panes fade into the void.

set -e

# 1. Dim entire pane (background + text), not just text.
#    With dark backgrounds this makes inactive panes visually recede.
defaults write com.googlecode.iterm2 DimOnlyText -bool false

# 2. Crank dimming from 0.6 to 0.85 — inactive panes become noticeably darker.
defaults write com.googlecode.iterm2 SplitPaneDimmingAmount -float 0.85

# 3. Subtle outline on the active tab in minimal mode.
defaults write com.googlecode.iterm2 MinimalTabStyleOutlineStrength -float 0.3

# 4. Slight background color difference for active tab.
defaults write com.googlecode.iterm2 MinimalTabStyleBackgroundColorDifference -float 0.08

echo "Done. Restart iTerm2 to apply."
