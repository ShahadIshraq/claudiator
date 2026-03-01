#!/bin/bash
set -e

# Extract version from git tag (e.g. ios-v1.2.3 → 1.2.3)
if [ -n "$CI_TAG" ]; then
    VERSION=$(echo "$CI_TAG" | sed 's/ios-v//')
    BUILD_NUMBER=$(git rev-list --count HEAD)

    # Update project.yml with tag version
    cd "$CI_PRIMARY_REPOSITORY_PATH/ios"
    sed -i '' "s/MARKETING_VERSION: .*/MARKETING_VERSION: \"$VERSION\"/" project.yml
    sed -i '' "s/CURRENT_PROJECT_VERSION: .*/CURRENT_PROJECT_VERSION: $BUILD_NUMBER/" project.yml

    # Regenerate project with updated version
    xcodegen generate

    echo "Version: $VERSION, Build: $BUILD_NUMBER"
fi
