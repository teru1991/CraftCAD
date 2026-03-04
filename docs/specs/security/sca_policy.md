# SCA Policy (Dependency Vulnerability Gate)

## Gate rule
- Any HIGH/CRITICAL vulnerability found in production dependencies blocks release.
- CI must fail on HIGH/CRITICAL by default.

## Exceptions (time-bounded only)
- Exceptions must be recorded with:
  - vulnerability id (e.g., RUSTSEC-XXXX-YYYY)
  - rationale
  - owner
  - expires_on (YYYY-MM-DD)
- Expired exceptions MUST fail CI.

## Tracking artifact
- The allowlist file lives under docs/specs/security/ (SSOT) and is reviewed like code.
- Format will be introduced alongside CI workflow in the SCA implementation task.
