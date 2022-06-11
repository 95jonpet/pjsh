#!/bin/bash
#
# Build linux packages from compiled binaries.

set -euo pipefail

VERSION="${1?'Missing required package version'}"
RELEASE="${2?'Missing required package release number'}"
SCRIPTS_PATH="${3?'Missing required path to install scripts'}"
RELEASE_PATH="${4?'Missing required path to compiled release'}"
PACKAGE_PATH="${5?'Missing required output path for packages'}"

# Use absolute paths.
SCRIPTS_PATH="$(realpath "${SCRIPTS_PATH}")"
RELEASE_PATH="$(realpath "${RELEASE_PATH}")"
PACKAGE_PATH="$(realpath "${PACKAGE_PATH}")"

if [[ ! -d "${RELEASE_PATH}" ]]; then
  echo "ERROR: The software has not been build. Nothing to package." >&2
  exit 1
fi

mkdir -p "${RELEASE_PATH}" "${PACKAGE_PATH}"

#######################################
# Cleanup files from the backup directory.
# Globals:
#   VERSION
#   RELEASE
# Arguments:
#   Package type, a string
#   Docker image with fpm, a string
#######################################
package() {
  local PACKAGE_TYPE="${1?'Missing required package type'}"
  local DOCKER_IMAGE="${2?'Missing required docker image'}"

  MSYS_NO_PATHCONV=1 docker run \
    --rm \
    -v "${SCRIPTS_PATH}:/scripts" \
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
    --after-install "/scripts/after-install.sh" \
    --before-remove "/scripts/before-remove.sh" \
    "/src/pjsh=/bin/pjsh"

  # Correct file ownership.
  MSYS_NO_PATHCONV=1 docker run \
    --rm \
    -v "${PACKAGE_PATH}:/out" \
    "${DOCKER_IMAGE}" \
    bash -c "chown '$(id -u):$(id -g)' /out/pjsh_${VERSION}-${RELEASE}.${PACKAGE_TYPE}"
}

# Build all packages.
# Separate container images are required due to differing fpm requirements.
package "deb" "alanfranz/fpm-within-docker:debian-bullseye"
package "rpm" "alanfranz/fpm-within-docker:centos-8"
package "tar" "alanfranz/fpm-within-docker:debian-bullseye"

# Compress the built .tar archive to a .tar.gz file using gzip.
find "${PACKAGE_PATH}" -name "*.tar" -exec gzip -v9 {} \;
