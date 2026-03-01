# BOM Rounding / Units Contract (v1)

- Numeric CSV fields are fixed precision according to implementation policy (`write_bom_csv`).
- Unit conversion follows document units policy; exported values must be deterministic.
- CSV encoding uses UTF-8 with BOM for spreadsheet compatibility.
- Delimiter is configurable (default comma) and escaping follows RFC-like CSV quoting rules.
- Row order is stable and deterministic.
