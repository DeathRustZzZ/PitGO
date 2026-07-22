#!/bin/sh

set -eu

ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR"

echo "Updating Graphify knowledge graph..."

graphify . --code-only
graphify cluster-only .

echo "Graphify knowledge graph updated."
