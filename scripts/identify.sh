#!/bin/bash

BASEDIR=$(dirname "$0")
. $BASEDIR/isotp-config.sh

isotp_send "11 01" # reset
wait_for_response "51"
isotp_send "10 03" # diag session
wait_for_response "50"
isotp_send "22 f1 00" # identification
wait_for_response "62 f1 00"
isotp_send "22 f1 5a" # fingerprint
wait_for_response "62 f1 5a"
isotp_send "22 f1 5b" # fingerprint history
wait_for_response "62 f1 5b"
isotp_send "22 f1 54" # hardware supplier
wait_for_response "62 f1 54"
isotp_send "22 f1 50" # hardware version
wait_for_response "62 f1 50"
isotp_send "22 f1 11" # hardware part number
wait_for_response "62 f1 11"
isotp_send "22 f1 53" # boot software version
wait_for_response "62 f1 53"
isotp_send "22 f1 55" # softawre supplier ID
wait_for_response "62 f1 55"
isotp_send "22 f1 21" # software part numbers
wait_for_response "62 f1 21"
isotp_send "22 02 00" # continental software versions
wait_for_response "62 02 00"
