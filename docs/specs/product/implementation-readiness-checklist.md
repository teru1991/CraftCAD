# Implementation Readiness Checklist (Gate to Start Feature Work)

## A) SSOT completeness
- [ ] `vision.md` exists and is consistent with existing docs/specs/**
- [ ] `feature-scope.md` defines Desktop/Mobile split and non-goals
- [ ] `contracts/master-model.md` defines Part/Material/FeatureGraph and stable IDs
- [ ] `contracts/derived-artifacts.md` defines derived artifacts + mobile packaging
- [ ] `contracts/jobs.md` defines canonical jobs + invariants
- [ ] `contracts/reason-codes.md` defines minimum ReasonCodes
- [ ] `contracts/rules-wood.md` defines rule checks + severities

## B) Testability
- [ ] `testing/e2e-flows.md` defines Flow A/B/C with concrete assertions
- [ ] Each new implementation task must cite which flow(s) it advances and adds/updates tests accordingly.

## C) Determinism & Safety
- [ ] Determinism policy referenced (../determinism/)
- [ ] Any heuristic uses seed and stable ordering
- [ ] Timeouts/cancel points are specified for heavy jobs

## D) Traceability
- [ ] docs/status/trace-index.json includes this task with artifact list
- [ ] docs/verification/P00-PREP-SSOT-001.md includes history evidence and a “GO/NO-GO” verdict

## GO/NO-GO rule
Implementation starts only if all boxes above are checked in verification.
