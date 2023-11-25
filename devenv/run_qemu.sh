#!/bin/bash -Eeu

# Set up logging
LIGHT_BLUE='\033[1;34m'
RED='\033[0;31m'
RESET='\033[0m'

# exec 2> >(awk '{print strftime("%H:%M:%S ['${RED}'ERROR'${RESET}'] ") $0;  system("");}')
# exec 1> >(awk '{print strftime("%H:%M:%S ['${LIGHT_BLUE}'INFO'${RESET}'] ") $0; system("");}')

# Set up variables
DEVENV_DIR=$(dirname "$0")
EFI_FILE=$1
ANOTHER_FILE=${2:-}
DIST_DIR=$DEVENV_DIR/../dist
DISK_IMG=$DIST_DIR/disk.img
MOUNT_POINT=$DIST_DIR/mnt

# Create disk image
$DEVENV_DIR/make_dist.sh $DIST_DIR
$DEVENV_DIR/make_image.sh $DISK_IMG $MOUNT_POINT $EFI_FILE $ANOTHER_FILE
$DEVENV_DIR/run_image.sh $DISK_IMG $DIST_DIR
