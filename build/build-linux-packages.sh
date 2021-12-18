#!/bin/bash

set -euo pipefail

VERSION="${1?'Missing required package version'}"
RELEASE="${2?'Missing required package release number'}"

DOCKER_IMAGE="alanfranz/fpm-within-docker:debian-bullseye"
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
RELEASE_PATH="${SCRIPT_DIR}/../target/release"
PACKAGE_PATH="${SCRIPT_DIR}/../target/package"

if [[ ! -d "${RELEASE_PATH}" ]]; then
  echo "ERROR: The software has not been build. Nothing to package." >&2
  exit 1
fi

package() {
  local PACKAGE_TYPE="${1?'Missing required package type'}"
  DOCKER_IMAGE="${2?'Missing required docker image'}"

  MSYS_NO_PATHCONV=1 docker run \
    --rm \
    -v "${RELEASE_PATH}:/src" \
    -v "${PACKAGE_PATH}:/out" \
    "${DOCKER_IMAGE}" \
    fpm \
    -s dir \
    --output-type "${PACKAGE_TYPE}" \
    --package "/out/pjsh_${VERSION}-${RELEASE}.${PACKAGE_TYPE}" \
    --name pjsh \
    --version "${VERSION}" \
    --iteration "${RELEASE}" \
    --architecture all \
    --license mit \
    --description "A non-POIX shell for the modern age" \
    --url "https://peterjonsson.se/shell" \
    --maintainer "Peter Jonsson" \
    "/src/pjsh=/usr/bin/pjsh"
}

# Build all packages.
# Separate container images are required due to differing fpm requirements.
package "deb" "alanfranz/fpm-within-docker:debian-bullseye"
package "rpm" "alanfranz/fpm-within-docker:centos-8"
package "tar" "alanfranz/fpm-within-docker:debian-bullseye"
