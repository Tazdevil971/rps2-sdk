#!/bin/bash
set -e

# Retrieve root of the project
ROOT=$(realpath $(dirname "$0")/..)

# Some variables to control
PCSX2_VERSION=v1.7.5724
PCSX2_EXE="pcsx2-$PCSX2_VERSION-linux-appimage-x64-Qt.AppImage"
PCSX2_URL="https://github.com/PCSX2/pcsx2/releases/download/$PCSX2_VERSION/$PCSX2_EXE"

ELF_SYMLINK_TARGET=/tmp/rps2-pcsx2-bin.elf

# Create pcsx2 directory
[ -d $ROOT/.pcsx2 ] || mkdir $ROOT/.pcsx2

if [ ! -f $ROOT/.pcsx2/$PCSX2_EXE ]; then
    (cd $ROOT/.pcsx2
        wget $PCSX2_URL
        chmod +x $PCSX2_EXE)
fi

# Symlink the executable to a known position with ".elf" extension
ln -sf $(realpath "$1") $ELF_SYMLINK_TARGET

# Shift in first argument
shift

$ROOT/.pcsx2/$PCSX2_EXE \
    -batch \
    ${PCSX2_EXTRA_ARGS:-} \
    -gameargs "host:$ELF_SYMLINK_TARGET $*" \
    -- $ELF_SYMLINK_TARGET