#!/bin/bash -Eeu

if [ $# -lt 1 ]; then
    echo "Usage: $0 <image name>" 1>&2
    exit 1
fi

DISK_IMG=$1
DIST_DIR=$2
OVMF_CODE_FILE=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd
OVMF_VARS_FILE=$DIST_DIR/OVMF_VARS.fd

if [ ! -f $DISK_IMG ]
then
    echo "No such file: $DISK_IMG" 1>&2
    exit 1
fi

if [ ! -f $OVMF_VARS_FILE ]
then
    cp /usr/share/edk2-ovmf/x64/OVMF_VARS.fd $OVMF_VARS_FILE
    echo "Copied OVMF_VARS.fd to /copy/of/OVMF_VARS.fd"
fi



qemu-system-x86_64 \
    -m 1G \
    -drive if=pflash,format=raw,readonly=on,file=$OVMF_CODE_FILE \
    -drive if=pflash,format=raw,file=$OVMF_VARS_FILE \
    -drive if=ide,index=0,media=disk,format=raw,file=$DISK_IMG \
    -device nec-usb-xhci,id=xhci \
    -device usb-mouse -device usb-kbd \
    -monitor stdio \
    ${QEMU_OPTS:-}
