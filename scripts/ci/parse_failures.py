#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

CATEGORY_PATTERNS: dict[str, list[str]] = {
    "rust_compile_error": [r"\berror\[E\d{4}\]", r"could not compile", r"aborting due to \d+ previous error"],
    "clippy": [r"clippy", r"\bwarning: .*\n", r"-D warnings"],
    "rust_test_fail": [r"test result: FAILED", r"failures:", r"\d+ failed"],
    "link_error": [r"undefined reference", r"linker .* failed", r"cannot find -l"],
    "cmake_error": [r"CMake Error", r"Configuring incomplete", r"Generate step failed"],
    "qt_meta_object": [r"moc", r"Q_OBJECT", r"undefined reference to `vtable", r"AUTOMOC"],
    "ctest_fail": [r"[0-9]+% tests passed", r"The following tests FAILED", r"\*\*\*Failed"],
}

PRIORITY = ["fmt", "clippy", "test", "link", "desktop"]


def categorize(content: str, filename: str) -> list[str]:
    categories: list[str] = []
    lower_file = filename.lower()

    if "rust_fmt" in lower_file and ("diff" in content or "not formatted" in content):
        categories.append("rust_compile_error")

    for category, patterns in CATEGORY_PATTERNS.items():
        if any(re.search(pattern, content, flags=re.IGNORECASE | re.MULTILINE) for pattern in patterns):
            categories.append(category)

    if not categories:
        if "[FAIL]" in content:
            categories.append("rust_compile_error")
        else:
            categories.append("none")

    return sorted(set(categories))


def recommend_priority(failure_categories: set[str]) -> str:
    if any(cat in failure_categories for cat in {"rust_compile_error"}):
        return "fmt"
    if "clippy" in failure_categories:
        return "clippy"
    if "rust_test_fail" in failure_categories:
        return "test"
    if "link_error" in failure_categories:
        return "link"
    if any(cat in failure_categories for cat in {"cmake_error", "qt_meta_object", "ctest_fail"}):
        return "desktop"
    return "none"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--log-dir", default=".ci_logs")
    parser.add_argument("--out", default=".ci_logs/summary.json")
    args = parser.parse_args()

    log_dir = Path(args.log_dir)
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    logs = sorted(log_dir.glob("*.log"))
    failures = []
    all_categories: set[str] = set()

    for log in logs:
        content = log.read_text(encoding="utf-8", errors="ignore")
        failed = "[FAIL]" in content
        categories = categorize(content, log.name)
        if failed:
            filtered_categories = [c for c in categories if c != "none"]
            all_categories.update(filtered_categories)
            failures.append(
                {
                    "log": log.name,
                    "categories": filtered_categories or ["rust_compile_error"],
                    "first_error_line": next(
                        (line for line in content.splitlines() if "error" in line.lower() or "failed" in line.lower()),
                        "",
                    ),
                }
            )

    summary = {
        "total_logs": len(logs),
        "total_failures": len(failures),
        "failures": failures,
        "priority_order": PRIORITY,
        "suggested_first_fix": recommend_priority(all_categories),
    }

    out_path.write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
