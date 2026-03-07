#!/usr/bin/env python3
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TRACE = ROOT / "docs" / "status" / "trace-index.json"

def fail(msg: str) -> None:
    print(f"[trace-check] ERROR: {msg}", file=sys.stderr)
    sys.exit(1)

def main() -> None:
    if not TRACE.exists():
        fail(f"missing {TRACE}")
    try:
        data = json.loads(TRACE.read_text(encoding="utf-8"))
    except Exception as e:
        fail(f"failed to parse trace-index.json: {e}")

    tasks = data.get("tasks")
    if not isinstance(tasks, dict):
        fail("trace-index.json must contain object field: tasks")

    used_ver = {}
    for task_id, entry in sorted(tasks.items(), key=lambda kv: kv[0]):
        if not isinstance(entry, dict):
            fail(f"tasks[{task_id}] must be an object")
        ver = entry.get("verification")
        if not isinstance(ver, str) or not ver:
            fail(f"tasks[{task_id}].verification must be non-empty string")
        ver_path = ROOT / ver
        if not ver_path.exists():
            fail(f"tasks[{task_id}].verification file missing: {ver}")
        if ver in used_ver:
            fail(f"verification referenced twice: {ver} by {used_ver[ver]} and {task_id}")
        used_ver[ver] = task_id

        artifacts = entry.get("artifacts")
        if not isinstance(artifacts, list) or len(artifacts) == 0:
            fail(f"tasks[{task_id}].artifacts must be a non-empty list")
        for a in artifacts:
            if not isinstance(a, str) or not a:
                fail(f"tasks[{task_id}].artifacts contains non-string/empty value")
            ap = ROOT / a
            if not ap.exists():
                fail(f"tasks[{task_id}] artifact missing: {a}")

    print(f"[trace-check] OK tasks={len(tasks)} verifications={len(used_ver)}")

if __name__ == "__main__":
    main()
