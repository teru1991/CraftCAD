# Migration policy

- 読み込みは N-2 まで保証。
- 書き出しは常に最新 schema_version。
- forward incompatible は ReadOnly で開き、ReasonCode を返す。
- migration は段階適用（vN->vN+1）。
- `--dry-run` では差分要約のみを表示し、ファイルは更新しない。
- `--in-place` は明示指定時のみ許可（既定は拒否）。
