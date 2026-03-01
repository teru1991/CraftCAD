# Versioning / v1.0 Freeze

## v1.0 Frozen Contracts
- ReasonCodes (`core/errors/reason_codes.json`)
- JSON schema files (`core/serialize/schemas/*.json`)
- Desktop FFI C ABI symbols and envelope format
- BOM CSV columns/order/rounding behavior
- Export option semantics and invariants

## Breaking change definition
A change is **breaking** in v1.x if it does any of the following:
- rename/remove existing schema field, or make optional->required
- rename/remove/change signature of exported FFI symbol
- reorder/rename existing BOM columns
- alter meaning of existing export options

## Safe evolution in v1.x
- add-only schema fields with defaults/normalization
- add-only optional FFI functions
- add BOM columns only at tail with documented defaults
- add export options as optional keys with stable defaults

## v2 policy
If a breaking change is needed, create migration policy and bump `schema_version`; do not silently change v1 behavior.
