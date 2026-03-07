# E2E Flows SSOT (Product Acceptance)

## Purpose
Pin “usable product” by end-to-end flows that must remain green.
These flows are the acceptance criteria for the 6 pillars.

## Flow A — First 15 minutes success (Desktop)
1. Open sample project
2. Select a part → change a key dimension (parameter)
3. Verify:
   - 3D updates
   - 2D drawing sheet updates (projection + at least one dimension)
   - BOM estimate updates
4. Run nesting job (confirm layout)
5. Export drawing (PDF) and nesting sheet
6. Save project → reopen → verify artifacts persist

### Assertions
- No silent failures; failures must produce ReasonCodes.
- Determinism: repeated run with same seed yields identical nesting + drawing hashes.
- Optional (projection-lite enabled): front/top/side projection hashes remain stable pre/post save+load for same SSOT snapshot.

## Flow B — Screw feature integration
1. Add ScrewFeature (line pattern) on a part edge
2. Verify:
   - screw markers + note appear on 2D sheet (or designated note list)
   - BOM fastener count increments
   - rules check catches edge distance violation when forced too close (FATAL)

## Flow C — Mobile read-only
1. On desktop: generate viewer pack artifacts and save
2. On mobile: open project offline
3. Verify:
   - 3D view loads
   - 2D sheets load
   - nesting sheet + BOM visible
   - if artifact missing, UI shows “Not generated” (no recompute)

## Link to existing testing SSOT
- ../testing/

## Acceptance note
- Determinism gate must be green for every PR for lite artifacts and viewpack coverage: projection_lite / estimate_lite / fastener_bom_lite / mfg_hints_lite / viewpack_v1.
- On mismatch, harness must emit a failure bundle at `build/determinism_failures/<run_id>/` including input SSOT, per-run artifacts/hashes, and environment.

## CI Desktop Gate
- Qt6 が利用可能な CI 環境では、Desktop build + 全 smoke（view3d / projection_lite / estimate_lite / fastener_bom_lite / rules_edge / mfg_hints_lite / inspector_edit）を固定順で実行し、各 smoke のログ由来 JSON 検証まで成功することを必須ゲートとする。
- Qt6 が利用できない環境では、Desktop gate は理由付きで `SKIP` として明示し、CI 全体は継続する。
- Desktop 判定・実行・SKIP 判定の SSOT は `scripts/ci/run_all.sh` とし、workflow 側に重複ロジックを持たない。

## Failure Artifacts Contract
- All CI runs must upload `build/ci_artifacts/**` regardless of success/failure (`if: always()`).
- Each failing step writes `build/ci_artifacts/<step_name>/` containing captured stdout/stderr and best-effort diagnostics bundle inputs.
- `build/ci_artifacts/index.json` is always generated and lists collected files per step.

