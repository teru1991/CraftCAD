# Golden Assets

期待値ファイル一覧：

- `geom/`: 幾何演算テスト期待値
- `nesting/`: 木取り（ネスティング）JSON 期待値
- `export/`: IO エクスポート期待値
- `io_roundtrip/`: import/export の round-trip 期待値

## Golden Update の安全な実行方法

### ステップ 1: 差分確認（推奨）

まず、テスト出力と期待値の差分を確認します：

```bash
cd C:\Users\Keiji\StudioKeKe\CraftCAD
cargo run --manifest-path tools/golden_update/Cargo.toml -- --dataset geom
```

出力：
- ✓ geom exists at tests/golden/geom/expected.json
- ✓ Validation complete

この時点では何も変更されません。

### ステップ 2: バックアップ付きで更新

変更が正しいことを確認したら、`--write` フラグで更新します：

```bash
cargo run --manifest-path tools/golden_update/Cargo.toml -- --dataset geom --write
```

このコマンドは：
1. 既存ファイルを `.backup.YYYYMMDD_HHMMSS` として自動保存
2. Golden ファイルを更新
3. 元に戻すコマンドをコンソールに表示

### ステップ 3: 複数 Dataset の更新

すべての Golden ファイルを更新する場合：

```bash
cargo run --manifest-path tools/golden_update/Cargo.toml -- --dataset all --write
```

**重要**: 複数 dataset を同時に更新する場合でも、各 dataset 単独での `--write` を推奨します。

### ステップ 4: 変更をコミット

差分を確認してからコミットします：

```bash
git diff tests/golden/
git add tests/golden/
git commit -m "Update golden files for [reason]"
```

## トラブルシューティング

### 予期しない差分が発生した場合

1. **テストコードの不具合を疑う**
   - 期待値ではなく、テストロジック自体が間違っていないか確認

2. **バックアップから復旧**
   ```bash
   ls tests/golden/*/*.backup.*
   # 最新のバックアップを確認して復旧
   cp tests/golden/geom/expected.json.backup.20260301_120000 tests/golden/geom/expected.json
   ```

3. **Git から復旧**
   ```bash
   git checkout tests/golden/
   ```

## CI での Golden Update

**注意**: CI/CD パイプラインでは `--write` フラグの使用は禁止されています。

Golden ファイルの更新が必要な場合は、ローカルで実行してコミットしてください。

## ファイル構成

```
tests/golden/
├── README.md                          # このファイル
├── geom/
│   ├── expected.json                  # 幾何演算期待値
│   └── expected.json.backup.*         # 自動バックアップ
├── nesting/
│   ├── expected_*.json                # ネスティング期待値
│   └── expected_*.json.backup.*
├── export/
│   ├── expected_*.json                # エクスポート期待値
│   └── expected_*.json.backup.*
└── inputs/                             # テストの入力ファイル
```
