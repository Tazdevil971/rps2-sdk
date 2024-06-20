#!/bin/bash
set -e
PCSX2=$(dirname "$0")/pcsx2.sh

EXE=$1
shift

PCSX2_EXTRA_ARGS="-debugger" \
$PCSX2 $EXE $@