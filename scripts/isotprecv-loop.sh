#!/bin/bash

BASEDIR=$(dirname "$0")

. $BASEDIR/isotp-config.sh

RUST_BACKTRACE=full ./target/release/isotprecv \
  -s $REQUEST_ARBITRATION_ID \
  -d $REPLY_ARBITRATION_ID \
  -p $TX_PADDING_BYTE:$RX_PADDING_BYTE \
  -f $ST_MIN \
  -l \
  $CAN_INTERFACE_NAME | tee isotp-buffer.txt
