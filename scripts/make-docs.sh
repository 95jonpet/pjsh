#!/bin/bash
#
# Compile or serve the external documentation.

set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
DOCS_DIR="${SCRIPT_DIR}/../doc"

COMMAND="${1:-build}"

cd -- "${DOCS_DIR}"

mdbook "${COMMAND}"
