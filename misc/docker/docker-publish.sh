#!/usr/bin/env bash
set -euo pipefail

# Usage: ./misc/docker/docker-publish.sh <dockerhub-user> [version]
#
# If version is omitted, it reads the version from Cargo.toml
# (the workspace root).  Example:
#
#   ./misc/docker/docker-publish.sh johndoe 0.1.0
#   ./misc/docker/docker-publish.sh johndoe       # reads version from Cargo.toml

DOCKER_USER="${1:?Usage: $0 <dockerhub-user> [version]}"
VERSION="${2:-}"

if [ -z "$VERSION" ]; then
    echo "No version specified; reading version from app/Cargo.toml …"
    VERSION="$(grep '^version = ' app/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"
fi

IMAGE="fiche-n-chips"
REMOTE="${DOCKER_USER}/${IMAGE}"

echo "==> Building ${REMOTE}:latest …"
docker build -f misc/docker/Dockerfile -t "${REMOTE}:latest" .

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
