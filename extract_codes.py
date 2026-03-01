#!/usr/bin/env python3
import json
import sys

# Load catalog.json
with open('docs/specs/errors/catalog.json', 'r', encoding='utf-8') as f:
    catalog = json.load(f)

# Extract all codes
codes = sorted([item['code'] for item in catalog['items']])

# Output as JSON (without BOM, with UTF-8)
output = {
    "codes": codes
}

# Write to file without BOM
with open('core/errors/reason_codes.json', 'w', encoding='utf-8') as f:
    json.dump(output, f, indent=2, ensure_ascii=False)
    f.write('\n')

print("Updated core/errors/reason_codes.json with {} codes".format(len(codes)))

