#!/bin/bash

BASEDIR=$(dirname "$0")

. $BASEDIR/isotp-config.sh

echo $1

printf "%s" $1 | ./target/release/isotpsend \
  -s $REQUEST_ARBITRATION_ID \
  -d $REPLY_ARBITRATION_ID \
  -p $TX_PADDING_BYTE:$RX_PADDING_BYTE \
  -f $ST_MIN \
  $CAN_INTERFACE_NAME
