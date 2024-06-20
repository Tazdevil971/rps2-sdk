#!/bin/bash
set -e

# First step is to unbug dsnetm
dd if=/dev/zero of=/tmp/rps2-zeroes.bin bs=1 count=2048
dsedb bload /tmp/rps2-zeroes.bin 0x100000

# Retrieve the executable name
EXE=$1
shift

# Then actually run the program
dsedb run $EXE $@