#!/bin/bash
# migrate-skills.sh - Move old skill files into new references/ directories
# and clean up old category directories

set -euo pipefail

SKILLS="/Users/rch/repo/dotfiles/.claude/skills"

echo "=== Moving protogen references ==="
cp "$SKILLS/design/proto-overview.md"      "$SKILLS/protogen/references/overview.md"
cp "$SKILLS/design/proto-schema.md"        "$SKILLS/protogen/references/schema.md"
cp "$SKILLS/design/proto-architecture.md"  "$SKILLS/protogen/references/architecture.md"
cp "$SKILLS/design/proto-database.md"      "$SKILLS/protogen/references/database.md"
cp "$SKILLS/design/proto-patterns.md"      "$SKILLS/protogen/references/patterns.md"
cp "$SKILLS/design/proto-project.md"       "$SKILLS/protogen/references/project.md"
cp "$SKILLS/design/proto-testing.md"       "$SKILLS/protogen/references/testing.md"
cp "$SKILLS/design/proto-frontend.md"      "$SKILLS/protogen/references/frontend.md"
cp "$SKILLS/design/proto-pitfalls.md"      "$SKILLS/protogen/references/pitfalls.md"

echo "=== Moving documentation references ==="
cp "$SKILLS/documentation/doc-overview.md"          "$SKILLS/documentation/references/overview.md"
cp "$SKILLS/documentation/doc-process.md"           "$SKILLS/documentation/references/process.md"
cp "$SKILLS/documentation/doc-content.md"           "$SKILLS/documentation/references/content.md"
cp "$SKILLS/documentation/doc-templates.md"         "$SKILLS/documentation/references/templates.md"
cp "$SKILLS/documentation/doc-verify.md"            "$SKILLS/documentation/references/verify.md"
cp "$SKILLS/documentation/doc-learnings.md"         "$SKILLS/documentation/references/learnings.md"
cp "$SKILLS/documentation/doc-organization.md"      "$SKILLS/documentation/references/organization.md"
cp "$SKILLS/documentation/documentation_merging.md" "$SKILLS/documentation/references/merging.md"
cp "$SKILLS/documentation/layered_documentation.md" "$SKILLS/documentation/references/layered.md"
cp "$SKILLS/documentation/marketing_lens.md"        "$SKILLS/documentation/references/marketing-lens.md"
cp "$SKILLS/documentation/rap_documentation.md"     "$SKILLS/documentation/references/rap.md"
cp "$SKILLS/documentation/tone_matrixing.md"        "$SKILLS/documentation/references/tone-matrixing.md"

echo "=== Moving project-process references ==="
cp "$SKILLS/meta/project-index.md"          "$SKILLS/project-process/references/index.md"
cp "$SKILLS/meta/project-artifacts.md"      "$SKILLS/project-process/references/artifacts.md"
cp "$SKILLS/meta/project-practices.md"      "$SKILLS/project-process/references/practices.md"
cp "$SKILLS/meta/project-priorities.md"     "$SKILLS/project-process/references/priorities.md"
cp "$SKILLS/meta/project-organization.md"   "$SKILLS/project-process/references/organization.md"
cp "$SKILLS/meta/project-multisubproject.md" "$SKILLS/project-process/references/multisubproject.md"
cp "$SKILLS/meta/PROVERBS.md"               "$SKILLS/project-process/references/proverbs.md"

echo "=== Fixing humanizer (move WARP.md to references/) ==="
cp "$SKILLS/humanizer/WARP.md" "$SKILLS/humanizer/references/warp.md"

echo "=== Moving old category directories ==="
# Create old/ archive for the category directories
mkdir -p "$SKILLS/old/categories"

# Move each category directory (preserving as archive)
for dir in default design engineering codebase utility github meta; do
  if [ -d "$SKILLS/$dir" ]; then
    cp -r "$SKILLS/$dir" "$SKILLS/old/categories/$dir"
    echo "  Archived: $dir -> old/categories/$dir"
  fi
done

# Now remove the old category directories
for dir in default design engineering codebase utility github meta; do
  if [ -d "$SKILLS/$dir" ]; then
    rm -rf "$SKILLS/$dir"
    echo "  Removed: $dir"
  fi
done

# Remove old flat files from documentation/ (now they live in references/)
rm -f "$SKILLS/documentation/doc-overview.md"
rm -f "$SKILLS/documentation/doc-process.md"
rm -f "$SKILLS/documentation/doc-content.md"
rm -f "$SKILLS/documentation/doc-templates.md"
rm -f "$SKILLS/documentation/doc-verify.md"
rm -f "$SKILLS/documentation/doc-learnings.md"
rm -f "$SKILLS/documentation/doc-organization.md"
rm -f "$SKILLS/documentation/documentation_merging.md"
rm -f "$SKILLS/documentation/layered_documentation.md"
rm -f "$SKILLS/documentation/marketing_lens.md"
rm -f "$SKILLS/documentation/rap_documentation.md"
rm -f "$SKILLS/documentation/tone_matrixing.md"

# Remove humanizer violations
rm -f "$SKILLS/humanizer/README.md"
rm -f "$SKILLS/humanizer/WARP.md"

# Remove routing table
rm -f "$SKILLS/README.md"

echo "=== Migration complete ==="
echo ""
echo "New structure:"
ls -la "$SKILLS/" | grep "^d"
echo ""
echo "Skill folders with SKILL.md:"
for dir in "$SKILLS"/*/; do
  if [ -f "$dir/SKILL.md" ]; then
    name=$(basename "$dir")
    refs=""
    if [ -d "$dir/references" ]; then
      count=$(ls "$dir/references/" 2>/dev/null | wc -l | tr -d ' ')
      refs=" (+ $count references)"
    fi
    echo "  $name$refs"
  fi
done
