#!/usr/bin/env python3
import json
import sys
import zipfile
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: create_view3d_smoke_fixture.py <out_path>", file=sys.stderr)
        return 2

    out = Path(sys.argv[1])
    out.parent.mkdir(parents=True, exist_ok=True)

    manifest = {
        "schema_version": "0",
        "app_version": "0.1.0",
        "units": "mm",
        "created_at": "2026-03-01T00:00:00Z",
        "modified_at": "2026-03-01T00:00:00Z",
    }
    data = {"entities": []}
    ssot = {
        "ssot_version": 1,
        "materials": [
            {
                "material_id": "00000000-0000-0000-0000-000000000001",
                "category": "unspecified",
                "name": "unspecified",
                "thickness_mm": 18.0,
                "grain_policy": "none",
                "kerf_mm": 2.0,
                "margin_mm": 5.0,
                "estimate_loss_factor": None,
            }
        ],
        "parts": [
            {
                "part_id": "00000000-0000-0000-0000-0000000000a1",
                "name": "smoke_part",
                "material_id": "00000000-0000-0000-0000-000000000001",
                "quantity": 1,
                "manufacturing_outline_2d": {
                    "min_x": 0.0,
                    "min_y": 0.0,
                    "max_x": 120.0,
                    "max_y": 80.0,
                },
                "thickness_mm": 18.0,
                "grain_direction": None,
                "labels": [{"key": "generated", "value": "true"}],
                "feature_ids": [],
            }
        ],
        "feature_graph": {"features": []},
    }

    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        zf.writestr("manifest.json", json.dumps(manifest, indent=2))
        zf.writestr("data.json", json.dumps(data, indent=2))
        zf.writestr("ssot_v1.json", json.dumps(ssot, indent=2))

    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
