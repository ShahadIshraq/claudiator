#!/bin/bash
set -e

# Install XcodeGen (Xcode Cloud has Homebrew)
brew install xcodegen

# Generate Xcode project from project.yml
cd "$CI_PRIMARY_REPOSITORY_PATH/ios"
xcodegen generate

echo "Project generated successfully"
