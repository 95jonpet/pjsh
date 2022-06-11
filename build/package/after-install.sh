#!/bin/bash
#
# Script to run after installing pjsh.

PJSH_PATH=/bin/pjsh

# Make pjsh a valid login shell.
if command -v add-shell &> /dev/null; then
  add-shell "${PJSH_PATH}"
fi
