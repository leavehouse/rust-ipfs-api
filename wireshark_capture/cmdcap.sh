#!/bin/sh
CAP_FILE_NAME="wireshark-$1-ipfs-api-$2.pcapng"
# TODO: I can't get this to work if I directly capture to /home/$USER.
WRITE_DIR="/tmp"
WRITE_FILE="$WRITE_DIR/$CAP_FILE_NAME"
FINAL_FILE="$PWD/$CAP_FILE_NAME"
trap endcap 2
endcap() {
    echo "Caught SIGINT ..."
    echo "Moving $WRITE_FILE to here ..."
    sudo mv $WRITE_FILE $PWD
    echo "Chowning $FINAL_FILE to $USER ..."
    sudo chown $USER $FINAL_FILE
    echo "Exiting ..."
    exit 1
}
sudo dumpcap -i lo -w $WRITE_FILE
