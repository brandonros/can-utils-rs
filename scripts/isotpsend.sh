#!/bin/bash

BASEDIR=$(dirname "$0")
. $BASEDIR/isotp-config.sh

isotp_send "$1"
