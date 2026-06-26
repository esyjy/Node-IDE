#!/usr/bin/env bash
set -euo pipefail

# Generates latest.json for Tauri updater from release artifacts.
# Usage: ./scripts/generate-update-manifest.sh <version> <notes> <artifact-url> <signature>

VERSION="${1:?version required}"
NOTES="${2:-}"
URL="${3:?artifact url required}"
SIGNATURE="${4:?signature required}"
PLATFORM="${5:-darwin-aarch64}"

cat <<EOF
{
  "version": "${VERSION}",
  "notes": "${NOTES}",
  "pub_date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "platforms": {
    "${PLATFORM}": {
      "url": "${URL}",
      "signature": "${SIGNATURE}"
    }
  }
}
EOF
