#!/bin/bash
REQUEST_ARBITRATION_ID="7E5"
REPLY_ARBITRATION_ID="7ED"
CAN_INTERFACE_NAME="ws://127.0.0.1:9001"
ST_MIN="2000000" # 2ms in nanoseconds
TX_PADDING_BYTE="55"
RX_PADDING_BYTE="AA"
[[ "$(uname)" = "Windows_NT" ]] && ISOTPSEND_PATH="./isotpsend.exe" || ISOTPSEND_PATH="./isotpsend"
[[ "$(uname)" = "Windows_NT" ]] && ISOTPRECV_PATH="./isotprecv.exe" || ISOTPRECV_PATH="./isotprecv"

isotp_send() {
  echo "$1"
  printf "%s" "$1" | $ISOTPSEND_PATH \
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
