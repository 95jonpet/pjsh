#!/bin/bash
#
# Build linux packages from compiled binaries.

set -euo pipefail

PACKAGE_PATH="${1?'Missing required output path for packages'}"
EXAMPLE_PATH="${2?'Missing required input path for examples'}"

# Use absolute paths.
EXAMPLE_PATH="$(realpath "${EXAMPLE_PATH}")"
PACKAGE_PATH="$(realpath "${PACKAGE_PATH}")"

# Common docker arguments.
DOCKER_ARGS=(
  --rm
  -v "${PACKAGE_PATH}:/packages:ro"
  -v "${EXAMPLE_PATH}:/examples:ro"
)

#######################################
# Verify the packaging for a platform.
# Globals:
#   DOCKER_ARGS
# Arguments:
#   Package file extension, a string
#   Docker image, a string
#   Package manager name, a string
#   Optional package manager options
#######################################
verify() {
  local FILE_EXTENSION="${1?'Missing required package file extension'}"
  local DOCKER_IMAGE="${2?'Missing required docker image'}"
  local PACKAGE_MANAGER="${3?'Missing required package manager'}"
  shift 3

  bsep="======================================================================="
  ssep="-----------------------------------------------------------------------"

  echo
  echo "Verifying .${FILE_EXTENSION} package using ${PACKAGE_MANAGER}..."
  echo "${bsep}"
  MSYS_NO_PATHCONV=1 docker run "${DOCKER_ARGS[@]}" "${DOCKER_IMAGE}" bash -c "
    set -euo pipefail

    # Install pjsh.
    find /packages \
      -name '*.${FILE_EXTENSION}' \
      -exec '${PACKAGE_MANAGER}' install -y $* {} \;

    # Run all examples using pjsh.
    # find /examples -name '*.pjsh' -exec pjsh -- {} \;
    for example in /examples/*.pjsh; do
      echo
      echo \"Running \${example}:\"
      echo '${ssep}'
      pjsh -- \"\${example}\"
      echo
    done
  "
  echo "${bsep}"
  echo
}

verify deb debian:stable-slim apt-get
verify rpm fedora dnf --cacheonly "--disablerepo=*"
