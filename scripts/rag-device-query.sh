#!/bin/bash
# Query the device-library RAG for port specifications.
# Usage: rag-device-query.sh <source-name> [query]
#
# Default query searches for connectors, ports, inputs, outputs.
# Uses the device-library-rag server on port 8086.
#
# Examples:
#   rag-device-query.sh yamaha-cl5
#   rag-device-query.sh digico-sd7 "expansion card slots"
#   rag-device-query.sh --sources

RAG_HOST="${RAG_HOST:-192.168.0.200}"
RAG_PORT="${RAG_PORT:-8086}"
BASE="http://${RAG_HOST}:${RAG_PORT}"

case "$1" in
    --sources) curl -sf "${BASE}/sources" && echo; exit ;;
    --help|-h) head -12 "$0" | tail -11; exit ;;
esac

if [ -z "$1" ]; then
    echo "Usage: rag-device-query.sh <source-name> [query]" >&2
    exit 1
fi

SOURCE="$1"
QUERY="${2:-connectors inputs outputs XLR BNC ports}"
TOP_N="${3:-8}"

ENCODED=$(python3 -c "import urllib.parse, sys; print(urllib.parse.quote_plus(sys.argv[1]))" "$QUERY" 2>/dev/null || echo "$QUERY" | sed 's/ /+/g')
URL="${BASE}/search?q=${ENCODED}&n=${TOP_N}&mode=hybrid&source=${SOURCE}"

curl -sf "$URL"
