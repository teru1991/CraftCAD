# Recovery & Autosave contract (SSOT)

Policy
- Autosave snapshots are app-local (not inside .diycad).
- Autosave is atomic: write temp -> fsync(best-effort) -> rename.
- Only save when dirty.
- Keep last known good always.

Recommended defaults (app)
- autosave_interval_sec: 60
- max_generations: 20
- max_total_bytes: 2147483648 (2GiB)
- keep_last_good: true

Restore behavior
- List generations newest-first.
- Even if some generations are corrupted, others must still be listed (best-effort).
- Restore returns OpenResult-like warnings/read_only signals.

Crash-safety contract
- If crash happens during autosave:
  - the last good generation remains restorable
  - temp partial files may remain but must be ignored safely
