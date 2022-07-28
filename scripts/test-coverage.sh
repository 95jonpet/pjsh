#!/bin/bash
#
# Run tests and record test coverage.

set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
APP_DIR="$(realpath "${SCRIPT_DIR}/..")"

cd -- "${APP_DIR}"

cargo tarpaulin \
  --output-dir target \
  --out html
