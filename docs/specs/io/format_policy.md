# I/O Format Policy

- Deterministic ordering, epsilon and rounding are SSOT-driven.
- Unsupported or degraded conversion must emit reason codes from `support_matrix.json`.
- Importers/exporters are panic-free for untrusted input boundaries.
- JSON format is the canonical highest-fidelity round-trip representation.
- Preflight limits (bytes/entities/nesting depth) are mandatory for import/open/migrate flows.
