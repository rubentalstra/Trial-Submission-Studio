#!/bin/bash
set -euo pipefail

REPO="${1:-rubentalstra/Trial-Submission-Studio}"

gh issue list -R "$REPO" --state open --limit 1000
gh label list -R "$REPO" --limit 1000
