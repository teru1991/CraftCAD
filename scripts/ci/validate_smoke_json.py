#!/usr/bin/env python3
"""Validate smoke-step log contains expected token and emit normalized JSON."""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--log", required=True)
    p.add_argument("--expect-token", required=True)
    p.add_argument("--out", required=True)
    return p.parse_args()


def main() -> int:
    args = parse_args()
    log_path = Path(args.log)
    if not log_path.is_file():
        raise SystemExit(f"log file not found: {log_path}")

    lines = log_path.read_text(encoding="utf-8", errors="replace").splitlines()
    matched = [line for line in lines if args.expect_token in line]
    if not matched:
        raise SystemExit(f"expected token not found: {args.expect_token}")

    line = matched[-1]
    kv_pairs: dict[str, str] = {}
    for m in re.finditer(r"([A-Za-z0-9_]+)=([^\s]+)", line):
        kv_pairs[m.group(1)] = m.group(2)

    payload = {
        "ok": True,
        "token": args.expect_token,
        "line": line,
        "fields": kv_pairs,
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    # Parse once more to guarantee valid JSON artifact.
    json.loads(out_path.read_text(encoding="utf-8"))
    print(f"smoke-json-ok: {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
