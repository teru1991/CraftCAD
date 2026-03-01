# BOM CSV/TSV Format

## Columns (fixed order)
1. `part_id`
2. `part_name`
3. `qty`
4. `material_name`
5. `thickness`
6. `bbox_w`
7. `bbox_h`
8. `area`
9. `perimeter`
10. `grain_dir`
11. `allow_rotate`
12. `margin`
13. `kerf`

## Deterministic sorting
- Primary: `material_name` ascending (byte-stable string compare)
- Secondary: `thickness` ascending numeric
- Then: `part_name` ascending
- Then: `part_id` ascending

## Units and rounding
- If `doc.units == "mm"`: lengths in mm, area in mm².
- If `doc.units == "inch"`: lengths in inch, area in inch².
- Length fields: round to 2 decimals.
- Area field: round to 2 decimals.
- Angle (`grain_dir`) exported as degrees, rounded to 1 decimal.

## CSV policy (Excel-safe)
- UTF-8 with BOM (`EF BB BF`)
- delimiter: comma
- escaping: RFC4180
- line endings: CRLF

## Example row
`f0f...,Panel A,2,Birch Ply,18.00,600.00,300.00,180000.00,1800.00,90.0,false,2.00,0.20`
