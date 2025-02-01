#!/bin/bash

LISTEN_PORT=8888
DURATION=10
TEMP_FILE=$(mktemp)

timeout "$DURATION" nc -u -l "$LISTEN_PORT" > "$TEMP_FILE" 2>/dev/null &
NC_PID=$!
sleep "$DURATION"
kill "$NC_PID" 2>/dev/null

TOTAL_BYTES_RECEIVED=$(stat --format="%s" "$TEMP_FILE")
ELAPSED_TIME=$DURATION

BITRATE=$(echo "scale=2; ($TOTAL_BYTES_RECEIVED * 8) / $ELAPSED_TIME" | bc)

echo "Total bytes received: $TOTAL_BYTES_RECEIVED"
echo "Time elapsed: $ELAPSED_TIME seconds"
echo "Bitrate: $BITRATE bps"

rm -f "$TEMP_FILE"
