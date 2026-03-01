# CraftCAD リポジトリ概要

CraftCAD の初期ブートストラップ構成です。clone 直後に迷わず開発を開始できるよう、主要ディレクトリと運用前提を整理しています。

## 基本方針

- **DB は利用しない**（SQLite を含む RDB/NoSQL を導入しない）。
- **正本フォーマットは `.diycad`** とする。
  - `.diycad` は `zip + manifest + data + assets` の構造を持つ想定。
- 技術スタック:
  - **Core**: Rust
  - **Desktop**: Qt
  - **Mobile**: Flutter

## ディレクトリ構造

```text
.
├─ core/                         # Rust 製ドメイン/計算コア
├─ apps/
│  ├─ desktop/                   # Qt デスクトップアプリ
│  └─ mobile/                    # Flutter モバイルアプリ
├─ tools/                        # 開発補助スクリプト/CI 補助
├─ docs/
│  └─ specs/
│     ├─ system/                 # システム仕様
│     ├─ security/               # セキュリティ仕様
│     ├─ observability/          # 監視/ロギング仕様
│     └─ release/                # リリース仕様
└─ testdata/                     # テスト用固定データ
```

## 開発コマンド（予定）

> まだ実体はありません。各サブプロジェクトの初期化後に提供予定です。

### Core（Rust）

```bash
# 予定
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test --all
```

### Desktop（Qt / CMake）

```bash
# 予定
cmake -S apps/desktop -B build/desktop
cmake --build build/desktop
ctest --test-dir build/desktop
```

### Mobile（Flutter）

```bash
# 予定
flutter pub get
flutter analyze
flutter test
```

## 初期セットアップ完了条件

- clone 直後に上記ディレクトリが揃っていること。
- リポジトリ方針（DB 不使用、`.diycad` 正本、技術スタック）が README で把握できること。


## v1.0 契約仕様（SSOT）

- `docs/specs/system/versioning.md`
- `docs/specs/system/ffi_contract.md`
- `docs/specs/system/schema_contract.md`
- `docs/specs/geom/edge_cases.md`
- `docs/specs/nesting/edge_cases.md`
- `docs/specs/part_bom/rounding_units.md`
- `docs/specs/export/contracts.md`

