#!/usr/bin/env bash
# Runs release-it without a package.json: npx fetches release-it and its
# plugins (@release-it/bumper, @release-it/conventional-changelog) into a
# shared, cached, ephemeral install and runs the CLI from there. Config
# lives in .release-it.json; the version itself lives only in Cargo.toml.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

exec npx --yes \
  -p release-it@21 \
  -p @release-it/bumper@8 \
  -p @release-it/conventional-changelog@12 \
  release-it "$@"
