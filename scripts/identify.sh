#!/bin/bash

BASEDIR=$(dirname "$0")
. $BASEDIR/isotp-config.sh

isotp_send() {
  echo $1
  printf "%s" $1 | ./target/release/isotpsend \
    -s $REQUEST_ARBITRATION_ID \
    -d $REPLY_ARBITRATION_ID \
    -p $TX_PADDING_BYTE:$RX_PADDING_BYTE \
    -f $ST_MIN \
    $CAN_INTERFACE_NAME
}

wait_for_response() {
  EXPECTED_RESPONSE=$1
  SIZE=$(printf "%s" "$EXPECTED_RESPONSE" | wc -c | tr -d ' ')
  while [ 1 ]
  do
    RESPONSE=$(tail -n 1 isotp-buffer.txt | head -c $SIZE)
    if [ "$RESPONSE" == "$EXPECTED_RESPONSE" ]
    then
      break
    fi
    echo "waiting... RESPONSE = \"$RESPONSE\" EXPECTED_RESPONSE = \"$EXPECTED_RESPONSE\" SIZE = $SIZE"
    sleep 0.1
  done
}

isotp_send "11 01" # reset
wait_for_response "51"
isotp_send "10 03" # diag session
wait_for_response "50"
isotp_send "22 f1 00" # identification
wait_for_response "62"
isotp_send "22 f1 5a" # fingerprint
wait_for_response "62"
isotp_send "22 f1 5b" # fingerprint history
wait_for_response "62"
isotp_send "22 f1 54" # hardware supplier
wait_for_response "62"
isotp_send "22 f1 50" # hardware version
wait_for_response "62"
isotp_send "22 f1 11" # hardware part number
wait_for_response "62"
isotp_send "22 f1 53" # boot software version
wait_for_response "62"
isotp_send "22 f1 55" # softawre supplier ID
wait_for_response "62"
isotp_send "22 f1 21" # software part numbers
wait_for_response "62"
isotp_send "22 02 00" # continental software versions
wait_for_response "62"
