#!/bin/bash
# Create GitHub Release for gity

REPO="shedrackgodstime/gity"
VERSION="v1.0.0"
TOKEN="${GITHUB_TOKEN}"

if [ -z "$TOKEN" ]; then
    echo "Error: Set GITHUB_TOKEN environment variable"
    echo "gh auth login"
    exit 1
fi

# Create release
RELEASE_JSON=$(curl -s -X POST https://api.github.com/repos/${REPO}/releases \
  -H "Authorization: token ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "tag_name": "'${VERSION}'",
    "name": "gity '${VERSION}'",
    "body": "## Features\n- Multiple SSH keys management\n- Passphrase protection\n- File permissions (600) enforcement\n- Security audit command\n- Host key verification\n- Key rotation\n- Cross-platform installation scripts\n\n## Installation\n```bash\ncurl -fsSL gity.pages.dev/install | sh\n```",
    "draft": false,
    "prerelease": false
  }')

UPLOAD_URL=$(echo "$RELEASE_JSON" | grep -o '"upload_url":"[^"]*' | cut -d'"' -f4)

echo "Release created: $UPLOAD_URL"
echo ""
echo "Upload binaries with: gh release upload ${VERSION} <files>"
echo ""
echo "Or manually at: https://github.com/${REPO}/releases/new?tag=${VERSION}"