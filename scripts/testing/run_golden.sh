#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

export CRAFTCAD_FAILURE_ARTIFACTS_DIR="${ROOT_DIR}/failure_artifacts"

ACCEPT="false"
TAGS="smoke"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --accept)
      ACCEPT="true"
      shift
      ;;
    --tags)
      TAGS="${2:-smoke}"
      shift 2
      ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

if [[ "${ACCEPT}" == "true" ]]; then
  echo "[run_golden] running golden_update --accept (LOCAL ONLY)"
  cargo run --manifest-path tools/golden_update/Cargo.toml -- --dataset all --write
fi

echo "[run_golden] checking manifest-referenced fixtures are text-only"
python - <<'PY'
from pathlib import Path
import json
root = Path('.')
manifest = json.loads((root / 'tests/datasets/manifest.json').read_text())
allowed = {'.json', '.svg', '.dxf'}
bad = []
for ds in manifest.get('datasets', []):
    for k in ('inputs', 'expected'):
        for entry in ds.get(k, []):
            p = root / entry['path']
            if not p.exists():
                bad.append(f'missing: {p}')
                continue
            if p.suffix.lower() not in allowed:
                bad.append(f'bad extension: {p}')
                continue
            if b'\x00' in p.read_bytes():
                bad.append(f'nul byte: {p}')
if bad:
    print('\n'.join(bad))
    raise SystemExit(1)
print('text-only fixture check: ok')
PY

echo "[run_golden] running golden dataset comparisons (NO GENERATION)"
cargo test --manifest-path core/Cargo.toml -p ssot_lint --test golden_datasets -- --nocapture
