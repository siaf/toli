#!/bin/bash

# Exit on error
set -e

# Check if version argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

VERSION=$1

# Build the release
cargo build --release

# Create release assets directory
rm -rf release-assets
mkdir -p release-assets

# Create temporary directories for packaging
rm -rf temp-darwin temp-linux
mkdir -p temp-darwin temp-linux

# Copy files for macOS package
cp target/release/toli temp-darwin/
cp -r completions temp-darwin/

# Copy files for Linux package
cp target/release/toli temp-linux/
cp -r completions temp-linux/

# Package for macOS
tar czf "release-assets/toli-${VERSION}-x86_64-apple-darwin.tar.gz" -C temp-darwin .

# Package for Linux (assuming cross-compilation is set up)
tar czf "release-assets/toli-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" -C temp-linux .

# Clean up temporary directories
rm -rf temp-darwin temp-linux

# Calculate SHA256 checksums
DARWIN_SHA256=$(shasum -a 256 "release-assets/toli-${VERSION}-x86_64-apple-darwin.tar.gz" | cut -d' ' -f1)
LINUX_SHA256=$(shasum -a 256 "release-assets/toli-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" | cut -d' ' -f1)

# Update Formula/toli.rb with new version and checksums
sed -i '' \
    -e "s/version \".*\"/version \"${VERSION}\"/" \
    -e "s/sha256 \".*\"/sha256 \"${DARWIN_SHA256}\"/" \
    -e "s/sha256 \".*\"/sha256 \"${LINUX_SHA256}\"/" \
    Formula/toli.rb

echo "
Release preparation completed!

Next steps:
1. Create a new GitHub release with tag v${VERSION}
2. Upload the following files to the release:
   - release-assets/toli-${VERSION}-x86_64-apple-darwin.tar.gz
   - release-assets/toli-${VERSION}-x86_64-unknown-linux-gnu.tar.gz
3. The Homebrew formula has been updated with the new version and checksums

Make sure to review the changes before committing!
"