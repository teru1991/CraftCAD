#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

bash "${ROOT_DIR}/scripts/build_desktop.sh"
bash "${ROOT_DIR}/scripts/run_desktop.sh" --smoke-resources
