#!/bin/bash

# see: https://stackoverflow.com/a/246128/128240
export TVTRACK_ROOT=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

export TVTRACK_CONFIG_FILE="${TVTRACK_ROOT}/data/tvtrack.test.config.json"

# see: https://stackoverflow.com/a/53214779
cargo run --bin tvtrack -- "${@}"
