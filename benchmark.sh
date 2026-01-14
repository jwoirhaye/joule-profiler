#!/usr/bin/env bash
# benchmark_polling.sh

set -e

BIN=./target/debug/joule-profiler
WORKLOAD=./cpu_burn.sh
DURATION=3

OUTDIR=bench_results
mkdir -p "$OUTDIR"

for POLL in $(seq 1000 1000 30000); do
    echo "===> Testing polling = ${POLL} ms"

    sleep 1
    START=$(date +%s.%N)

    sudo $BIN \
        simple \
        --jouleit-file $OUTDIR/poll_${POLL}.json \
        --json \
        --rapl-polling "$POLL" \
        -- \
        "$WORKLOAD" "$DURATION"

    EXIT_CODE=$?

    END=$(date +%s.%N)
    ELAPSED=$(echo "$END - $START" | bc)

    echo "$POLL,$EXIT_CODE,$ELAPSED" >> "$OUTDIR/results.csv"

    if [ "$EXIT_CODE" -ne 0 ]; then
        echo "❌ Failure at polling=$POLL"
        break
    fi
done

echo "Benchmark finished"
