#!/usr/bin/env python3
"""Generate stable index.json for build/ci_artifacts."""
from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


@dataclass
class StepEntry:
    name: str
    files: list[str]
    bytes_total: int


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser()
    p.add_argument("--artifacts-dir", default="build/ci_artifacts")
    return p.parse_args()


def collect_steps(artifacts_dir: Path) -> list[StepEntry]:
    steps: list[StepEntry] = []
    if not artifacts_dir.exists():
        return steps

    for child in sorted(p for p in artifacts_dir.iterdir() if p.is_dir()):
        files: list[str] = []
        bytes_total = 0
        for fp in sorted(p for p in child.rglob("*") if p.is_file()):
            rel = fp.relative_to(child).as_posix()
            files.append(rel)
            bytes_total += fp.stat().st_size
        steps.append(StepEntry(name=child.name, files=files, bytes_total=bytes_total))
    return steps


def main() -> int:
    args = parse_args()
    artifacts_dir = Path(args.artifacts_dir)
    artifacts_dir.mkdir(parents=True, exist_ok=True)

    steps = collect_steps(artifacts_dir)
    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "steps": [
            {
                "name": step.name,
                "files": step.files,
                "bytes_total": step.bytes_total,
            }
            for step in steps
        ],
    }

    out_path = artifacts_dir / "index.json"
    out_path.write_text(json.dumps(payload, ensure_ascii=False, sort_keys=False, indent=2) + "\n", encoding="utf-8")
    print(out_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
