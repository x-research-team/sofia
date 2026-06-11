#!/bin/bash
# scripts/install-hooks.sh — установить git hooks для проекта

HOOKS_DIR="$(cd "$(dirname "$0")/.." && pwd)/.git/hooks"

cat > "$HOOKS_DIR/post-commit" << 'HOOK'
#!/bin/bash
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$PROJECT_ROOT" || exit 0
CHANGED_FILES=$(git diff --name-only HEAD~1..HEAD 2>/dev/null || echo "")
if echo "$CHANGED_FILES" | grep -qE '\.rs$|Cargo\.toml|Cargo\.lock|docs/'; then
    python3 scripts/update_readme.py --trigger=post-commit >> .readme_sync.log 2>&1
fi
HOOK
chmod +x "$HOOKS_DIR/post-commit"
echo "Hook installed: $HOOKS_DIR/post-commit"
