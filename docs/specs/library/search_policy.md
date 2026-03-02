# Library Search Policy

## Query tokenize
- lower
- trim
- 連続空白圧縮
- 記号は分割（`_` と `-` は分割するが元も保持）

## スコア優先順位（大→小）
1. id完全一致
2. id前方一致
3. display_name/タイトル完全一致
4. display_name/タイトル部分一致
5. tags一致
6. kind一致（wood/leather/process/output/hardware/template）

## tie-break（決定性）
- version desc（SemVer）
- source priority: builtin > user > project（将来）
