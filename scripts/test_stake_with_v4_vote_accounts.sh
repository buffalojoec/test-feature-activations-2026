#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

FILE="${1:-$SCRIPT_DIR/out/vote_v4_accounts_localnet.txt}"
NUM_ACCOUNTS="${2:-10}"
NETWORK="${3:-localnet}"

if [[ ! -f "$FILE" ]]; then
    echo "Error: File not found: $FILE"
    echo "Usage: $0 [file] [num_accounts] [network]"
    exit 1
fi

echo "Building stake binary..."
cargo build --bin simd-0185-stake --features simd-0185/bin --quiet

echo "Testing $NUM_ACCOUNTS random accounts from $FILE"

SUCCESS=0
FAILURE=0

for vote_account in $(shuf -n "$NUM_ACCOUNTS" "$FILE"); do
    echo ""
    echo "Testing: $vote_account"
    if cargo run --bin simd-0185-stake --features simd-0185/bin --quiet -- "$NETWORK" "$vote_account"; then
        SUCCESS=$((SUCCESS + 1))
    else
        FAILURE=$((FAILURE + 1))
    fi
done

echo ""
echo "Done. Success: $SUCCESS, Failed: $FAILURE"
