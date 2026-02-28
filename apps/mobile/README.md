# Mobile (Flutter)

`apps/mobile` は CraftCAD のモバイル向け **軽量ビューア** の最小プロジェクトです。

- 現時点の対象は閲覧（Viewer）中心です。
- 編集機能は後続 Issue で追加します。
- `.diycad` 読み込みや Rust FFI 連携も後続 Issue で対応します。

## 実行手順（flutter run）

1. Flutter SDK をインストールし、`flutter doctor` で環境を確認する。
2. このディレクトリへ移動する。

```bash
cd apps/mobile
flutter pub get
flutter run
```

初期画面には `DIY CAD Viewer` のテキストのみを表示します。
