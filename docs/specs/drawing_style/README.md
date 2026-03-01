# drawing_style SSOT 運用ルール（唯一の正）

## 1. ここにあるもの
- style_ssot.*：寸法/線種/フォント/丸め/線幅など“図面スタイル”のSSOT
- sheet_templates.*：A4/A3など用紙テンプレ（図枠/タイトル欄/表示領域）
- print_presets.*：印刷プリセット（fit_to_page、BW変換、線幅倍率、エクスポート設定）
- dimension_rules.md / annotation_rules.md：SSOTの解釈規約（実装が守る約束）

## 2. 禁止事項（最重要）
- 実装側に寸法・線幅・フォントサイズ等の定数を直書きしない
- 既存ID（*_vN）の内容を破壊的に編集しない（互換破壊・ゴールデン崩壊の原因）

## 3. 追加・変更手順（互換維持）
1) schemaは原則後方互換で拡張（optional追加）
2) 既存プリセットを変更したい場合：
   - 既存 preset/style/template は編集せず、新しいID（例：default_v2）を追加する
3) SSOT lint と Golden が必ず通ること
4) UIやプロジェクト側は参照IDを新IDへ切り替える（将来Task2/3で実装）

## 4. ID規約
- 形式：lower_snake_case_vN（例：a4_default_v1）
- vN は互換上の“見た目契約”の版。値が変わるなら v を上げる。

## 5. 期待する将来統合
- DrawingDoc は style_preset_id / sheet_template_id / print_preset_id を保持する（Task2）
- exporter がこのSSOTを読み、RenderIR→SVG/PDFへ変換する（Task3以降）
