#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:8080}"
ROOT="${2:-./artifacts/support_bundle}"
TS="$(date +%Y%m%d_%H%M%S)"
OUT_DIR="${ROOT}/${TS}"

mkdir -p "${OUT_DIR}"

echo "[collect] base=${BASE_URL}"
echo "[collect] out=${OUT_DIR}"

curl -fsS "${BASE_URL}/healthz" -o "${OUT_DIR}/healthz.json"
curl -fsS "${BASE_URL}/support_bundle" -o "${OUT_DIR}/support_bundle.json"
curl -fsS "${BASE_URL}/metrics" -o "${OUT_DIR}/metrics.prom"

cat > "${OUT_DIR}/manifest.txt" <<MANIFEST
base_url=${BASE_URL}
timestamp=${TS}
files:
- healthz.json
- support_bundle.json
- metrics.prom
MANIFEST

echo "[ok] collected:"
ls -la "${OUT_DIR}"
