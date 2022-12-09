#!/bin/bash
#
# Lint the code using clippy.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"

CLIPPY_FLAGS=(
  -W clippy::map-unwrap-or
  -W clippy::semicolon-if-nothing-returned
  -W clippy::single-match-else
)

cd "${SCRIPT_DIR}/.."
cargo clippy -- "${CLIPPY_FLAGS[@]}"
