#!/bin/bash

export TVTRACK_CONFIG_FILE="$PWD/data/tvtrack.config.json"

# see: https://stackoverflow.com/a/53214779
cargo run --bin tvtrack -- "${@}"
