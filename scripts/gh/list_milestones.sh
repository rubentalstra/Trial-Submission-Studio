#!/usr/bin/env bash
set -euo pipefail

REPO="rubentalstra/Trial-Submission-Studio"
HDR_ACCEPT="Accept: application/vnd.github+json"
HDR_VER="X-GitHub-Api-Version: 2022-11-28"

if ! command -v gh >/dev/null 2>&1; then
  echo "Error: 'gh' is not installed or not on PATH." >&2
  exit 1
fi

# Fetch milestones (open + closed), output TSV: number, title, state, open_issues, closed_issues, due_on
MILES_TSV="$(gh api -H "$HDR_ACCEPT" -H "$HDR_VER" \
  "repos/$REPO/milestones?state=all&per_page=100" --paginate \
  --jq '.[] | [.number, .title, .state, .open_issues, .closed_issues, (.due_on // "-")] | @tsv')"

COUNT=0
if [[ -n "$MILES_TSV" ]]; then
  COUNT="$(printf '%s\n' "$MILES_TSV" | wc -l | tr -d ' ')"
fi

echo "Showing ${COUNT} of ${COUNT} milestones in ${REPO}"
echo

print_table() {
  printf "ID\tTITLE\tSTATE\tOPEN\tCLOSED\tDUE\n"
  if [[ -n "$MILES_TSV" ]]; then
    printf '%s\n' "$MILES_TSV" | while IFS=$'\t' read -r num title state open closed due; do
      printf "#%s\t%s\t%s\t%s\t%s\t%s\n" "$num" "$title" "$state" "$open" "$closed" "$due"
    done
  fi
}

if command -v column >/dev/null 2>&1; then
  print_table | column -t -s $'\t'
else
  print_table
fi
