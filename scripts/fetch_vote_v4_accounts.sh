#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTDIR="$SCRIPT_DIR/out"
mkdir -p "$OUTDIR"

NETWORK="${1:-}"

case "$NETWORK" in
  mainnet)
    RPC_URL="https://api.mainnet-beta.solana.com"
    ;;
  devnet)
    RPC_URL="https://api.devnet.solana.com"
    ;;
  testnet)
    RPC_URL="https://api.testnet.solana.com"
    ;;
  localnet)
    RPC_URL="http://localhost:8899"
    ;;
  "")
    RPC_URL=$(solana config get | grep "RPC URL" | awk '{print $3}')
    ;;
  *)
    echo "Usage: $0 [mainnet|devnet|testnet|localnet]"
    exit 1
    ;;
esac

OUTFILE="$OUTDIR/vote_v4_accounts${NETWORK:+_$NETWORK}.txt"
echo "Using RPC URL: $RPC_URL"

# Vote program ID.
VOTE_PROGRAM="Vote111111111111111111111111111111111111111"

# Vote state V4 begins with LE u32 value 3 -> bytes [03, 00, 00, 00] -> base64 "AwAAAA==".
# Use dataSlice length 0 since we only need the pubkeys.
echo "Fetching V4 vote accounts from $VOTE_PROGRAM..."

curl -s "$RPC_URL" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getProgramAccounts",
    "params": [
      "'"$VOTE_PROGRAM"'",
      {
        "encoding": "base64",
        "dataSlice": { "offset": 0, "length": 0 },
        "filters": [
          {
            "memcmp": {
              "offset": 0,
              "bytes": "AwAAAA==",
              "encoding": "base64"
            }
          }
        ]
      }
    ]
  }' | jq -r '.result[].pubkey' > "$OUTFILE"

COUNT=$(wc -l < "$OUTFILE")
echo "Saved $COUNT V4 vote account addresses to $OUTFILE"
