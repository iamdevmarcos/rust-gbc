#!/bin/bash
export LIBRARY_PATH=/opt/homebrew/lib:$LIBRARY_PATH
cd "$(dirname "$0")"
cargo run --release "$@"

