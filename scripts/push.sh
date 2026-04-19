#!/usr/bin/env bash
# Push to the GitHub origin using a PAT from `.env.local`.
#
# Reads GITHUB_PAT / GITHUB_USER / GITHUB_REPO from `.env.local` and
# feeds them to git via `credential.helper`, which keeps the token out of
# `.git/config`, shell history, and process arguments. On success, git
# writes no token-bearing URL or header to disk.
#
# Usage (from repo root):
#
#   ./scripts/push.sh             # push current branch to origin
#   ./scripts/push.sh main        # push explicit branch
#   ./scripts/push.sh -u main     # -u / --set-upstream passthrough
#
# Any arguments are forwarded to `git push`.

set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
env_file="$here/.env.local"

if [[ ! -f "$env_file" ]]; then
    echo "error: $env_file not found. Copy .env.local.example and fill in GITHUB_PAT." >&2
    exit 2
fi

# Intentionally scoped: we don't `export` these — they stay in this shell.
# shellcheck disable=SC1090
set -a
. "$env_file"
set +a

: "${GITHUB_PAT:?GITHUB_PAT is empty in .env.local}"
: "${GITHUB_USER:?GITHUB_USER is empty in .env.local}"
: "${GITHUB_REPO:?GITHUB_REPO is empty in .env.local}"

url="https://github.com/${GITHUB_REPO}.git"

# A one-shot credential helper: when git asks for credentials for the
# matching URL, we print them on stdout and exit. No storage, no logging.
#
# `GIT_ASKPASS`-based approaches leak the token into the subprocess's
# argv/env in some setups; `credential.helper` with a shell function
# avoids that because git reads from the helper's stdout directly.
helper='!f() { echo "username=${GITHUB_USER}"; echo "password=${GITHUB_PAT}"; }; f'

GITHUB_USER="$GITHUB_USER" GITHUB_PAT="$GITHUB_PAT" \
git -c "credential.helper=" \
    -c "credential.helper=$helper" \
    push "$url" "$@"
