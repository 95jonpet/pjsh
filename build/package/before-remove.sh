#!/bin/bash
#
# Script to run before removing (uninstalling) pjsh.

PJSH_PATH=/bin/pjsh

# Make pjsh a valid login shell.
if command -v remove-shell &> /dev/null; then
  remove-shell "${PJSH_PATH}"
fi
