#!/usr/bin/env python3
import json

# Load support_matrix.json
with open('docs/specs/io/support_matrix.json', 'r', encoding='utf-8') as f:
    matrix = json.load(f)

# Extract all unique reason codes
codes = set()
for format_name, features in matrix['formats'].items():
    for feature_name, cell in features.items():
        for code in cell.get('reason_codes', []):
            if code:  # Skip empty strings
                codes.add(code)

# Also check mapping_rules.json
with open('docs/specs/io/mapping_rules.json', 'r', encoding='utf-8') as f:
    rules = json.load(f)
    if 'unit_rules' in rules and 'reason_code_on_assume' in rules['unit_rules']:
        codes.add(rules['unit_rules']['reason_code_on_assume'])

print("Missing IO codes from support_matrix and mapping_rules:")
for code in sorted(codes):
    print(f"  {code}")

