#!/bin/bash
#
# Check the code for formatting violations using rustfmt.

set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
APP_DIR="$(realpath "${SCRIPT_DIR}/..")"

cd -- "${APP_DIR}"

find . -name "*.rs" -type f -print0 | xargs -r -0 rustfmt --check --
