#!/bin/bash
# Create all project labels via GitHub CLI
# Usage: ./.github/scripts/create-labels.sh

set -e

echo "Creating GitHub labels for Trial Submission Studio..."

# Type labels
gh label create "bug" --color "d73a4a" --description "Something isn't working" --force
gh label create "enhancement" --color "a2eeef" --description "New feature or request" --force
gh label create "documentation" --color "0075ca" --description "Improvements or additions to documentation" --force
gh label create "standards" --color "7057ff" --description "CDISC standards compliance" --force
gh label create "performance" --color "fbca04" --description "Performance improvements" --force

# Status labels
gh label create "needs triage" --color "e99695" --description "New issue needs review" --force
gh label create "wontfix" --color "ffffff" --description "This will not be worked on" --force
gh label create "duplicate" --color "cfd3d7" --description "This issue or PR already exists" --force

# Community labels
gh label create "good first issue" --color "7057ff" --description "Good for newcomers" --force
gh label create "help wanted" --color "008672" --description "Help is requested to fix this issue" --force

# Automation labels
gh label create "automated" --color "ededed" --description "Automated PR from GitHub Actions" --force

# Platform labels
gh label create "windows" --color "0078d4" --description "Windows-specific issue" --force
gh label create "macos" --color "999999" --description "macOS-specific issue" --force
gh label create "linux" --color "666666" --description "Linux-specific issue" --force

# Standards labels
gh label create "SDTM" --color "1d76db" --description "Study Data Tabulation Model" --force
gh label create "ADaM" --color "5319e7" --description "Analysis Data Model" --force
gh label create "SEND" --color "0e8a16" --description "Standard for Exchange of Nonclinical Data" --force
gh label create "CT" --color "d93f0b" --description "Controlled Terminology" --force
gh label create "Define-XML" --color "c5def5" --description "Define-XML format issues" --force
gh label create "Dataset-XML" --color "bfd4f2" --description "Dataset-XML format issues" --force
gh label create "XPT" --color "fef2c0" --description "XPT transport format (V5/V8)" --force

echo ""
echo "All 21 labels created successfully!"
