#!/bin/bash
#
# Profile the application.
#
# Prerequisites:
#  - The "perf" program.
#  - The "rustfilt" crate.
#  - Flamegraph scripts from https://github.com/brendangregg/FlameGraph.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
TARGET_DIR="${SCRIPT_DIR}/../target"
PJSH="${TARGET_DIR}/release/pjsh"
EXAMPLE="$(realpath "${1?'Missing script to profile'}")"

WORKDIR="$(mktemp -d)"
trap 'rm -rf -- "${WORKDIR}"' EXIT

if [[ ! -f "${PJSH}" ]]; then
  cd "${SCRIPT_DIR}/../"
  CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release
fi

cd "${WORKDIR}"
sudo perf record --call-graph dwarf "${PJSH}" "${EXAMPLE}" > /dev/null
sudo chown "$(id -u):$(id -g)" perf.data

perf script \
  | stackcollapse-perf.pl \
  | rustfilt \
  | flamegraph.pl \
  > "${TARGET_DIR}/flame.svg"

cp perf.data "${TARGET_DIR}"
