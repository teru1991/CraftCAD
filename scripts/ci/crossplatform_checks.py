#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

EXEC_SCRIPTS = [
    "scripts/build_desktop.sh",
    "scripts/run_desktop.sh",
    "scripts/ci/run_all.sh",
    "scripts/ci/trace_index_check.py",
]
OPTIONAL_EXEC_SCRIPTS = [
    "scripts/ci/validate_smoke_json.py",
]

LF_SCAN_FILES = [
    "scripts/build_desktop.sh",
    "scripts/run_desktop.sh",
    "scripts/ci/run_all.sh",
    "scripts/ci/crossplatform_checks.py",
    "docs/specs/product/contracts/crossplatform.md",
]

BANNED_PATTERNS = [r"\breadlink\s+-f\b", r"\bsed\s+-i\b", r"\bapt-get\b"]
OS_GUARDS = ["runner.os", "uname", "OSTYPE", "if [ \"${OS_NAME}\"", "if [ \"$RUNNER_OS\"", "if: runner.os"]


def run(cmd: list[str]) -> str:
    return subprocess.check_output(cmd, cwd=ROOT, text=True, stderr=subprocess.STDOUT)


def git_mode(path: str) -> str | None:
    out = run(["git", "ls-files", "--stage", "--", path]).strip()
    if not out:
        return None
    return out.split()[0]


def check_executable_modes(errors: list[str]) -> None:
    for path in EXEC_SCRIPTS:
        mode = git_mode(path)
        if mode is None:
            errors.append(f"missing required script in git index: {path}")
            continue
        if mode != "100755":
            errors.append(f"script is not executable (expected 100755): {path} mode={mode}")

    for path in OPTIONAL_EXEC_SCRIPTS:
        mode = git_mode(path)
        if mode is not None and mode != "100755":
            errors.append(f"optional script present but not executable: {path} mode={mode}")


def has_crlf(path: Path) -> bool:
    return b"\r\n" in path.read_bytes()


def check_lf_only(errors: list[str]) -> None:
    for rel in LF_SCAN_FILES:
        p = ROOT / rel
        if not p.exists():
            errors.append(f"LF check target missing: {rel}")
            continue
        if has_crlf(p):
            errors.append(f"CRLF detected in LF-only file: {rel}")


def guarded_line(text: str, lineno: int) -> bool:
    lo = max(0, lineno - 4)
    hi = min(len(text.splitlines()), lineno + 3)
    chunk = "\n".join(text.splitlines()[lo:hi]).lower()
    return any(g.lower() in chunk for g in OS_GUARDS)


def check_linux_only_commands(errors: list[str]) -> None:
    for script in (ROOT / "scripts").rglob("*.sh"):
        text = script.read_text(encoding="utf-8", errors="replace")
        lines = text.splitlines()
        for pat in BANNED_PATTERNS:
            rx = re.compile(pat)
            for i, line in enumerate(lines, start=1):
                if rx.search(line):
                    if not guarded_line(text, i):
                        rel = script.relative_to(ROOT).as_posix()
                        errors.append(f"unguarded platform-specific command: {rel}:{i}: {line.strip()}")


def check_atomic_save_contract(errors: list[str]) -> None:
    target = ROOT / "core/crates/diycad_format/src/save.rs"
    if not target.exists():
        errors.append("atomic save source missing: core/crates/diycad_format/src/save.rs")
        return
    text = target.read_text(encoding="utf-8", errors="replace")
    needed = ["fs::rename", "SaveAtomicRenameFailed", ".tmp"]
    for n in needed:
        if n not in text:
            errors.append(f"atomic save signal not found: {n} in {target.relative_to(ROOT)}")


def unicode_path_probe(errors: list[str]) -> None:
    with tempfile.TemporaryDirectory(prefix="craftcad_") as td:
        root = Path(td)
        uni_dir = root / "テスト"
        uni_dir.mkdir(parents=True, exist_ok=True)
        file_path = uni_dir / "ファイル.txt"
        payload = "ok-ユニコード\n"
        file_path.write_text(payload, encoding="utf-8")
        got = file_path.read_text(encoding="utf-8")
        if got != payload:
            errors.append("unicode path probe read/write mismatch")


def run_self_test(errors: list[str]) -> None:
    with tempfile.TemporaryDirectory(prefix="crossplatform_selftest_") as td:
        p = Path(td) / "crlf.txt"
        p.write_bytes(b"a\r\nb\r\n")
        if not has_crlf(p):
            errors.append("self-test failed: CRLF detector did not detect CRLF")
        p2 = Path(td) / "lf.txt"
        p2.write_bytes(b"a\nb\n")
        if has_crlf(p2):
            errors.append("self-test failed: CRLF detector false positive on LF file")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    errors: list[str] = []
    check_executable_modes(errors)
    check_lf_only(errors)
    check_linux_only_commands(errors)
    check_atomic_save_contract(errors)
    unicode_path_probe(errors)
    if args.self_test:
        run_self_test(errors)

    if errors:
        print("[crossplatform_checks] FAIL")
        for e in errors:
            print(f" - {e}")
        return 1

    print("[crossplatform_checks] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
