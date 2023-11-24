#!/bin/bash -Eeu

DIST_DIR=$1

if [ ! -d "$DIST_DIR" ]; then
  mkdir -p "$DIST_DIR"
  echo "Created $DIST_DIR"
else
  echo "$DIST_DIR already exists. Skipping creation."
fi
