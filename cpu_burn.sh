#!/usr/bin/env bash
# cpu_burn.sh

set -e

DURATION=${1:-10}

END=$((SECONDS + DURATION))
while [ $SECONDS -lt $END ]; do
    echo "scale=500; a(1)*a(1)" | bc -l > /dev/null
done
