#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TRACE = ROOT / "docs/status/trace-index.json"


def main() -> int:
    if not TRACE.exists():
        print(f"trace-index missing: {TRACE}")
        return 1
    json.loads(TRACE.read_text(encoding="utf-8"))
    print("trace-index ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
