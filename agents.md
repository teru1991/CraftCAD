# Codex Red/Green Loop Rules

You MUST run tests after every meaningful change by executing:
- Linux/macOS: `scripts/ci/run_all.sh`
- Windows: `scripts/ci/run_all.ps1`

If tests fail:
1. Read `.ci_logs/summary.json` and the relevant failing log(s)
2. Fix the smallest root cause
3. Add or adjust regression tests if applicable
4. Re-run `scripts/ci/run_all.*` immediately
5. Repeat until everything passes

Never declare done unless `run_all` exits with status 0 and `.ci_logs/summary.json` reports zero failures.

Prefer minimal diffs, one fix per commit, and include why + a link to the failing log section.
