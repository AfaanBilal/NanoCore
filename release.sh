#!/usr/bin/env bash
set -euo pipefail

# Usage: ./release.sh <version>
# Example: ./release.sh 0.2.0

VERSION="${1-}"

if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

# Strip leading 'v' if provided
VERSION="${VERSION#v}"
TAG="v$VERSION"

# Validate semver format
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be in semver format (e.g. 1.2.3)"
  exit 1
fi

# Ensure working tree is clean
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Error: working tree is not clean — commit or stash changes first"
  exit 1
fi

# Ensure tag doesn't already exist
if [[ -n "$(git tag --list "$TAG")" ]]; then
  echo "Error: tag $TAG already exists"
  exit 1
fi

echo "Releasing $TAG..."

# Bump version in Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak

# Update Cargo.lock
cargo update --workspace --quiet

# Commit
git add Cargo.toml Cargo.lock docs/index.html
git commit -m "chore: release $TAG"

# Tag and push
git tag -a "$TAG" -m "Release $TAG"
git push origin HEAD
git push origin "$TAG"

# Publish to crates.io
cargo publish

echo "Done — $TAG pushed and published to crates.io. GitHub Actions will build and publish the release."
