#!/bin/bash

# Check if version argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 x.y.z"
    echo "Example: $0 1.0.7"
    exit 1
fi

NEW_VERSION=$1

# Validate version format
if ! echo "$NEW_VERSION" | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' > /dev/null; then
    echo "Error: Version must be in format x.y.z (e.g., 1.0.7)"
    exit 1
fi

# Update version in Cargo.toml
sed -i "s/^version = \".*\" #app-version/version = \"$NEW_VERSION\" #app-version/" Cargo.toml

# Stage the changes
git add Cargo.toml

# Commit the changes
git commit -m "chore: bump version to $NEW_VERSION"

# Create and push tag
git tag "v$NEW_VERSION"
git push origin main "v$NEW_VERSION"

echo "Released version $NEW_VERSION successfully!" 