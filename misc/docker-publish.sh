#!/usr/bin/env bash
set -euo pipefail

# Usage: ./misc/docker-publish.sh <dockerhub-user> [version]
#
# If version is omitted, it reads the version from Cargo.toml
# (the workspace root).  Example:
#
#   ./misc/docker-publish.sh johndoe 0.1.0
#   ./misc/docker-publish.sh johndoe           # reads version from Cargo.toml

DOCKER_USER="${1:?Usage: $0 <dockerhub-user> [version]}"
VERSION="${2:-}"

# If no version was supplied, try to extract it from the workspace Cargo.toml.
# The workspace Cargo.toml doesn't have a [package] section of its own, so
# we default to 0.1.0 for a first release.
if [ -z "$VERSION" ]; then
    echo "No version specified; defaulting to \"0.1.0\" for the first release."
    echo "To set a different version: $0 <user> <version>"
    VERSION="0.1.0"
fi

IMAGE="fiche-n-chips"
REMOTE="${DOCKER_USER}/${IMAGE}"

echo "==> Building ${REMOTE}:latest …"
docker build -f misc/Dockerfile -t "${REMOTE}:latest" .

echo "==> Tagging ${REMOTE}:v${VERSION} …"
docker tag "${REMOTE}:latest" "${REMOTE}:v${VERSION}"

echo "==> Pushing ${REMOTE}:latest …"
docker push "${REMOTE}:latest"

echo "==> Pushing ${REMOTE}:v${VERSION} …"
docker push "${REMOTE}:v${VERSION}"

echo ""
echo "✓ Done!  Published:"
echo "  ${REMOTE}:latest"
echo "  ${REMOTE}:v${VERSION}"
