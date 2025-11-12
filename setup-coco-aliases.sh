#!/bin/bash
# Setup git aliases for conventional commits (per-project)
# Run this in any project where you want to use CocoGitto

echo "Setting up git aliases for conventional commits..."

git config --local alias.feat '!git commit -m "feat: $1"'
git config --local alias.fix '!git commit -m "fix: $1"'
git config --local alias.docs '!git commit -m "docs: $1"'
git config --local alias.chore '!git commit -m "chore: $1"'
git config --local alias.refactor '!git commit -m "refactor: $1"'
git config --local alias.test '!git commit -m "test: $1"'
git config --local alias.perf '!git commit -m "perf: $1"'
git config --local alias.style '!git commit -m "style: $1"'
git config --local alias.ci '!git commit -m "ci: $1"'
git config --local alias.build '!git commit -m "build: $1"'

echo "✅ Git aliases configured for this project!"
echo ""
echo "Usage examples:"
echo "  git feat add new feature"
echo "  git fix resolve bug"
echo "  git docs update readme"
echo ""
echo "These aliases are stored in .git/config (per-project)"
