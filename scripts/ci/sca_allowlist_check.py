#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from datetime import date
from pathlib import Path


def fail(msg: str, code: int = 1) -> None:
    print(f"[sca-allowlist] ERROR: {msg}", file=sys.stderr)
    sys.exit(code)


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    allow = root / "docs" / "specs" / "security" / "sca_allowlist.json"
    if not allow.exists():
        fail(f"missing {allow}")
    try:
        data = json.loads(allow.read_text(encoding="utf-8"))
    except Exception as e:
        fail(f"failed to parse allowlist json: {e}")

    if not isinstance(data, dict):
        fail("allowlist must be object")
    if data.get("version", 0) < 1:
        fail("allowlist version must be >= 1")
    exc = data.get("exceptions")
    if exc is None:
        fail("missing exceptions")
    if not isinstance(exc, list):
        fail("exceptions must be array")

    today = date.today()
    seen = set()
    expired = []
    for i, it in enumerate(exc):
        if not isinstance(it, dict):
            fail(f"exceptions[{i}] must be object")
        for k in ["id", "rationale", "owner", "expires_on"]:
            if k not in it:
                fail(f"exceptions[{i}] missing key '{k}'")
        _id = it["id"]
        if not isinstance(_id, str) or len(_id.strip()) < 6:
            fail(f"exceptions[{i}].id invalid")
        if _id in seen:
            fail(f"duplicate exception id: {_id}")
        seen.add(_id)

        exp = it["expires_on"]
        if not isinstance(exp, str) or len(exp) != 10:
            fail(f"exceptions[{i}].expires_on invalid")
        try:
            y, m, d = [int(x) for x in exp.split("-")]
            exp_d = date(y, m, d)
        except Exception:
            fail(f"exceptions[{i}].expires_on parse failed: {exp}")

        if exp_d < today:
            expired.append((_id, exp))

    if expired:
        for _id, exp in expired:
            print(
                f"[sca-allowlist] EXPIRED: {_id} expired_on={exp} today={today.isoformat()}",
                file=sys.stderr,
            )
        return 2

    print(f"[sca-allowlist] OK: {len(exc)} exceptions, none expired.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
