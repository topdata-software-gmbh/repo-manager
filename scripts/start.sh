#!/usr/bin/env bash
#
# Launch the installed Git Repo Manager.
# 10/2026 created
#

set -euo pipefail

if BIN="$(command -v git-repo-manager 2>/dev/null)"; then
  :
elif [[ -n "${HOME:-}" && -x "${HOME}/.local/bin/git-repo-manager" ]]; then
  BIN="${HOME}/.local/bin/git-repo-manager"
else
  cat >&2 <<'EOF'
Git Repo Manager is not installed.

  Install it:    ./scripts/install.sh
  Desktop dev:   pnpm tauri dev
  Browser dev:   ./scripts/dev-up.sh
EOF
  exit 1
fi

exec "$BIN" "$@"
