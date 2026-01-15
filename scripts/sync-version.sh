#!/usr/bin/env bash
#
# sync-version.sh - Synchronize version across all metadata files
#
# Usage: ./scripts/sync-version.sh
#
# This script reads the version from Cargo.toml and updates:
# - CITATION.cff
# - .zenodo.json
# - codemeta.json
# - assets/linux/*.metainfo.xml (Flatpak/AppStream metadata)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Extract version from Cargo.toml workspace section
VERSION=$(grep -A5 '^\[workspace\.package\]' Cargo.toml | grep '^version' | head -1 | sed 's/version = "\(.*\)"/\1/')
DATE=$(date +%Y-%m-%d)

if [ -z "$VERSION" ]; then
    echo -e "${RED}Error: Could not extract version from Cargo.toml${NC}"
    exit 1
fi

echo -e "${YELLOW}Syncing version ${GREEN}$VERSION${YELLOW} (date: $DATE)${NC}"
echo ""

# Update CITATION.cff
echo -n "  Updating CITATION.cff... "
sed -i.bak "s/^version: .*/version: '$VERSION'/" CITATION.cff
sed -i.bak "s/^date-released: .*/date-released: '$DATE'/" CITATION.cff
rm -f CITATION.cff.bak
echo -e "${GREEN}done${NC}"

# Update .zenodo.json
echo -n "  Updating .zenodo.json... "
sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" .zenodo.json
rm -f .zenodo.json.bak
echo -e "${GREEN}done${NC}"

# Update codemeta.json
echo -n "  Updating codemeta.json... "
sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" codemeta.json
sed -i.bak "s/\"dateModified\": \"[^\"]*\"/\"dateModified\": \"$DATE\"/" codemeta.json
rm -f codemeta.json.bak
echo -e "${GREEN}done${NC}"

# Update metainfo.xml (Flatpak/AppStream)
METAINFO_FILE="assets/linux/io.github.rubentalstra.trial-submission-studio.metainfo.xml"
if [ -f "$METAINFO_FILE" ]; then
    echo -n "  Updating metainfo.xml... "
    # Update the most recent release version and date
    sed -i.bak "s/<release version=\"[^\"]*\" date=\"[^\"]*\">/<release version=\"$VERSION\" date=\"$DATE\">/" "$METAINFO_FILE"
    rm -f "${METAINFO_FILE}.bak"
    echo -e "${GREEN}done${NC}"
else
    echo -e "  ${YELLOW}Skipping metainfo.xml (file not found)${NC}"
fi

echo ""
echo -e "${GREEN}All files updated to version $VERSION${NC}"
echo ""

# Verify files are valid
echo "Validating files..."
if command -v python3 &> /dev/null; then
    python3 -m json.tool .zenodo.json > /dev/null && echo -e "  .zenodo.json: ${GREEN}valid${NC}"
    python3 -m json.tool codemeta.json > /dev/null && echo -e "  codemeta.json: ${GREEN}valid${NC}"
    python3 -c "import xml.etree.ElementTree as ET; ET.parse('$METAINFO_FILE')" 2>/dev/null && echo -e "  metainfo.xml: ${GREEN}valid${NC}"
else
    echo -e "  ${YELLOW}Skipping validation (python3 not found)${NC}"
fi

echo ""
echo -e "${GREEN}Version sync complete!${NC}"
