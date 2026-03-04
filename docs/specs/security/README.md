# Security & Privacy SSOT (Sprint18)

This directory is the single source of truth for:
- threat_model.md: scope/threats/mitigations
- limits.json: input limits (profiles)
- redaction_rules.json: redaction rules
- consent.md + consent.schema.json: consent contract
- sca_policy.md: SCA gate policy

## Rules
- Do not add new inputs/parsers without:
  - limits preflight coverage
  - sandbox rules (path/external refs)
  - redaction coverage where applicable
- Defaults must remain privacy-preserving (opt-in).

## How to change limits / rules
- Update JSON + keep schema compatible
- Run spec_ssot_lint tests
- Add/adjust unit tests for new rules
