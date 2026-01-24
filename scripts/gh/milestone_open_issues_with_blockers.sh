#!/usr/bin/env bash
set -euo pipefail

REPO="rubentalstra/Trial-Submission-Studio"
MS_NUM="${1:-}"
LIMIT="${2:-500}"

# Optional: cap how many blockers to print per group (0 = unlimited)
MAX_BLOCKERS="${MAX_BLOCKERS:-0}"

if [[ -z "$MS_NUM" || ! "$MS_NUM" =~ ^[0-9]+$ ]]; then
  echo "Usage: $0 <milestone-number> [limit]" >&2
  echo "Example: $0 1 500" >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "Error: 'gh' is not installed or not on PATH." >&2
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "Error: not authenticated. Run: gh auth login" >&2
  exit 1
fi

HEADERS=(
  -H "Accept: application/vnd.github+json"
  -H "X-GitHub-Api-Version: 2022-11-28"
)

repeat_char() {
  local ch="$1" n="$2"
  printf "%*s" "$n" "" | tr " " "$ch"
}

format_blockers_csv() {
  # Turns "1,2,3" into "#1,#2,#3" (+ optional limit)
  local csv="${1:-}"
  if [[ -z "$csv" ]]; then
    echo ""
    return
  fi

  IFS=',' read -r -a nums <<< "$csv"
  local total="${#nums[@]}"
  local show_total="$total"
  local suffix=""

  if (( MAX_BLOCKERS > 0 && total > MAX_BLOCKERS )); then
    show_total="$MAX_BLOCKERS"
    suffix=" (+$((total - MAX_BLOCKERS)) more)"
  fi

  local out=""
  for ((i=0; i<show_total; i++)); do
    [[ -z "${nums[$i]}" ]] && continue
    if [[ -n "$out" ]]; then out+=","
    fi
    out+="#${nums[$i]}"
  done

  echo "${out}${suffix}"
}

bold_on=$'\033[1m'
bold_off=$'\033[0m'

MS_TITLE="$(gh api "${HEADERS[@]}" "repos/$REPO/milestones/$MS_NUM" --jq .title 2>/dev/null || true)"
if [[ -z "$MS_TITLE" ]]; then
  echo "Error: milestone #$MS_NUM not found in $REPO (or no access)." >&2
  exit 1
fi

# Fetch ALL items in the milestone (issues endpoint includes PRs too)
# TSV columns: number, title, labels, state, is_pr
ITEMS_TSV="$(
  gh api "${HEADERS[@]}" \
    "repos/$REPO/issues?milestone=$MS_NUM&state=all&per_page=100" \
    --paginate \
    --jq '.[] | [
        (.number|tostring),
        (.title | gsub("[\r\n]+"; " ")),
        ([.labels[].name] | join(", ")),
        (.state),
        (if .pull_request? then "1" else "0" end)
      ] | @tsv'
)"

COUNT=0
if [[ -n "$ITEMS_TSV" ]]; then
  COUNT="$(printf '%s\n' "$ITEMS_TSV" | wc -l | tr -d ' ')"
fi

PRINT_COUNT="$COUNT"
if (( PRINT_COUNT > LIMIT )); then
  PRINT_COUNT="$LIMIT"
fi

echo "Showing ${PRINT_COUNT} of ${COUNT} items in ${REPO} that match your search"
echo "Milestone: #${MS_NUM} ${MS_TITLE}"
echo

# Compute widths (no truncation)
w_id=2
w_state=5
w_title=5
w_labels=6
w_blocked=10

rows=()
seen=0

while IFS=$'\t' read -r num title labels state is_pr; do
  [[ -z "${num:-}" ]] && continue
  (( seen >= LIMIT )) && break
  ((seen++))

  id="#$num"
  st="$(echo "$state" | tr '[:lower:]' '[:upper:]')"

  # For PRs: if closed and merged, show MERGED
  if [[ "$is_pr" == "1" && "$state" == "closed" ]]; then
    merged_at="$(gh api "${HEADERS[@]}" "repos/$REPO/pulls/$num" --jq '.merged_at // ""' 2>/dev/null || true)"
    if [[ -n "$merged_at" ]]; then
      st="MERGED"
    fi
  fi

  blockers_csv="$(
    gh api "${HEADERS[@]}" \
      "repos/$REPO/issues/$num/dependencies/blocked_by?per_page=100" \
      --jq '
        # Some endpoints return arrays, others wrap in {items:[]}/{nodes:[]}
        def items:
          if type=="array" then .
          elif has("items") then .items
          elif has("nodes") then .nodes
          else [] end;

        [ items[]? as $x
          | ($x.number // $x.issue.number // $x.content.number // empty) as $n
          | (($x.state // $x.issue.state // $x.content.state // "open") | ascii_downcase) as $s
          | select($n != null and $n != "")
          | select($s != "closed")
          | $n
        ] | join(",")
      ' 2>/dev/null || true
  )"

  if [[ -n "$blockers_csv" ]]; then
    blocked_by="#${blockers_csv//,/,#}"
  else
    blocked_by="-"
  fi

  rows+=("$id"$'\t'"$st"$'\t'"$title"$'\t'"${labels:-}"$'\t'"$blocked_by")

  (( ${#id} > w_id )) && w_id=${#id}
  (( ${#st} > w_state )) && w_state=${#st}
  (( ${#title} > w_title )) && w_title=${#title}
  (( ${#labels} > w_labels )) && w_labels=${#labels}
  (( ${#blocked_by} > w_blocked )) && w_blocked=${#blocked_by}
done <<< "$ITEMS_TSV"

# Header + underline
printf "%s%-*s  %-*s  %-*s  %-*s  %-*s%s\n" \
  "$bold_on" \
  "$w_id" "ID" \
  "$w_state" "STATE" \
  "$w_title" "TITLE" \
  "$w_labels" "LABELS" \
  "$w_blocked" "BLOCKED BY" \
  "$bold_off"

printf "%-*s  %-*s  %-*s  %-*s  %-*s\n" \
  "$w_id" "$(repeat_char "─" "$w_id")" \
  "$w_state" "$(repeat_char "─" "$w_state")" \
  "$w_title" "$(repeat_char "─" "$w_title")" \
  "$w_labels" "$(repeat_char "─" "$w_labels")" \
  "$w_blocked" "$(repeat_char "─" "$w_blocked")"

# Rows
for r in "${rows[@]}"; do
  IFS=$'\t' read -r id st title labels blocked_by <<< "$r"
  printf "%-*s  %-*s  %-*s  %-*s  %-*s\n" \
    "$w_id" "$id" \
    "$w_state" "$st" \
    "$w_title" "$title" \
    "$w_labels" "${labels:-}" \
    "$w_blocked" "$blocked_by"
done
