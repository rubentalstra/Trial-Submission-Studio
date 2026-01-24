#!/bin/bash
# Script to set up issue dependencies using GitHub's native "blocked by" feature
# Uses GraphQL addBlockedBy mutation

REPO="rubentalstra/Trial-Submission-Studio"

add_blocked_by() {
    local blocked=$1
    local blocker=$2
    echo -n "#$blocked blocked by #$blocker... "

    blocked_id=$(gh api graphql -f query="query { repository(owner: \"rubentalstra\", name: \"Trial-Submission-Studio\") { issue(number: $blocked) { id } } }" --jq '.data.repository.issue.id' 2>/dev/null)
    blocker_id=$(gh api graphql -f query="query { repository(owner: \"rubentalstra\", name: \"Trial-Submission-Studio\") { issue(number: $blocker) { id } } }" --jq '.data.repository.issue.id' 2>/dev/null)

    if [ -z "$blocked_id" ] || [ -z "$blocker_id" ]; then
        echo "SKIP (issue not found)"
        return
    fi

    gh api graphql -f query="mutation { addBlockedBy(input: {issueId: \"$blocked_id\", blockingIssueId: \"$blocker_id\"}) { clientMutationId } }" > /dev/null 2>&1 && echo "OK" || echo "FAILED"
    sleep 0.1
}
#
#echo "Setting up blocked-by relationships..."
#echo ""
#echo "NOTE: Dependencies referencing closed issues have been removed."
#echo "      Closed issues: 115, 119, 120, 121, 122, 142, 145, 148, 157, 161, 162, 164, 165, 168,"
#echo "                     174, 175, 178, 179, 180, 181, 182, 184, 185, 186, 194, 195, 204, 206,"
#echo "                     209, 218, 223, 225, 229, 231, 232, 233, 238, 239, 240, 253, 265, 267"
#echo ""
#
#echo "--- Standards ---"
## REMOVED: add_blocked_by 116 238  # #238 closed (ADaM metadata exists)
#add_blocked_by 116 243
## REMOVED: add_blocked_by 117 239  # #239 closed (SEND_CT exists)
#add_blocked_by 114 112
#
#echo "--- Normalization ---"
## REMOVED: add_blocked_by 115 113  # #115 closed (no USUBJID format validation)
#add_blocked_by 124 113
#
#echo "--- CT Chain ---"
## REMOVED: add_blocked_by 123 240  # #240 closed (no production unwrap)
## REMOVED: add_blocked_by 126 240  # #240 closed
## REMOVED: add_blocked_by 131 240  # #240 closed
## REMOVED: add_blocked_by 132 240  # #240 closed
## REMOVED: add_blocked_by 246 239  # #239 closed (SEND_CT exists)
## REMOVED: add_blocked_by 247 240  # #240 closed
## REMOVED: add_blocked_by 248 240  # #240 closed
## REMOVED: add_blocked_by 249 240  # #240 closed
#
#echo "--- Ingest ---"
#add_blocked_by 135 134
#add_blocked_by 136 134
## REMOVED: add_blocked_by 142 137  # #142 closed (delimiter detection claim invalid)
#
#echo "--- Persistence ---"
#add_blocked_by 144 143
## REMOVED: add_blocked_by 145 143  # #145 closed (atomic writes exist)
#add_blocked_by 146 143
#
#echo "--- GUI Error Handling ---"
#add_blocked_by 151 150
## REMOVED: add_blocked_by 164 151  # #164 closed (error boundary exists)
## REMOVED: add_blocked_by 168 151  # #168 closed (error recovery exists)
#
#echo "--- GUI Architecture ---"
#add_blocked_by 156 155
## REMOVED: add_blocked_by 157 155  # #157 closed (handlers return Task properly)
#add_blocked_by 158 155
#add_blocked_by 158 156
## REMOVED: add_blocked_by 158 157  # #157 closed
#add_blocked_by 167 155
#add_blocked_by 167 156
## REMOVED: add_blocked_by 167 157  # #157 closed
#
#echo "--- macOS ---"
## REMOVED: add_blocked_by 148 147  # #148 closed (OnceLock is atomic)
#
#echo "--- Cancellation ---"
## REMOVED: add_blocked_by 162 201  # #162 closed (cancellation propagated)
## REMOVED: add_blocked_by 162 202  # #162 closed
#
#echo "--- Performance ---"
## REMOVED: add_blocked_by 185 186  # Both #185 and #186 closed
#add_blocked_by 188 187             # Updated: #188 blocked by #187 (parallel export)
#add_blocked_by 189 187             # Updated: blocked by #187 instead of closed #186
#add_blocked_by 196 187             # Updated: blocked by #187 instead of closed #185
#
#echo "--- UX ---"
#add_blocked_by 213 158
## REMOVED: add_blocked_by 214 174  # #174 closed (confirmation dialogs exist)
## REMOVED: add_blocked_by 218 114  # #218 closed (recent files exist)
#add_blocked_by 219 173
#add_blocked_by 230 188
#
#echo "--- Testing ---"
#add_blocked_by 254 112
#add_blocked_by 254 134
#add_blocked_by 254 143
#add_blocked_by 255 260
#add_blocked_by 257 254
#add_blocked_by 258 254
#add_blocked_by 259 254
#
#echo "--- Documentation ---"
#add_blocked_by 133 130
#add_blocked_by 250 246
## REMOVED: add_blocked_by 251 238  # #238 closed (ADaM metadata exists)
#add_blocked_by 264 158
#add_blocked_by 266 133
## REMOVED: add_blocked_by 267 264  # #267 closed (architecture docs exist)

echo "--- Rust 2024 Best Practices (NEW) ---"
add_blocked_by 275 273             # Standardize Result handling blocked by fixing silent error suppression

echo ""
echo "Done! Check issues to see blocked-by relationships."
