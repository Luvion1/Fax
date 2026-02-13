#!/bin/bash
# Build and deploy documentation to gh-pages branch

set -e

echo "Building Jekyll site..."
cd docs
jekyll build --destination ../gh-pages-temp
cd ..

echo "Committing to gh-pages branch..."
git checkout gh-pages || git checkout -b gh-pages

# Remove old files except .git
find . -maxdepth 1 ! -name '.git' ! -name '.' ! -name '..' -exec rm -rf {} +

# Copy new files
cp -r gh-pages-temp/* .

# Clean up
rm -rf gh-pages-temp

# Commit and push
git add -A
git commit -m "docs: Update GitHub Pages" || echo "No changes to commit"
git push origin gh-pages

# Switch back to master
git checkout master

echo "Done! GitHub Pages should be updated."
