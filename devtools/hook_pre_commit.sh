#!/usr/bin/env zsh
# Claude Code PreToolUse hook: ensure release binary is built before any git commit.
# Receives JSON tool-input on stdin. If the bash command starts with `git commit`,
# runs the release build. Exit non-zero blocks the commit.

set -e

# Read tool input JSON from stdin
INPUT=$(cat)

# Extract the bash command (jq-free using grep/sed for portability)
COMMAND=$(printf '%s' "$INPUT" | python3 -c 'import sys, json; print(json.loads(sys.stdin.read()).get("tool_input", {}).get("command", ""))' 2>/dev/null || echo "")

# Only act on git commit commands
case "$COMMAND" in
    *"git commit"*)
        SCRIPT_DIR="$(dirname "$0")"
        if ! "$SCRIPT_DIR/build_release.sh" >&2; then
            echo "❌ Release build failed - blocking commit. Fix the build, then commit." >&2
            exit 1
        fi
        ;;
esac

# Always exit 0 to allow the tool call to proceed
exit 0
