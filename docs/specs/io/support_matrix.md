# I/O 対応表（DXF / SVG / JSON）

凡例: Supported / Best-effort / Not supported（Best-effortは劣化内容とReasonCode必須）

| Feature / Entity | DXF Import | DXF Export | SVG Import | SVG Export | JSON Import | JSON Export | Notes / ReasonCode |
|---|---|---|---|---|---|---|---|
| Line | Supported | Supported | Supported | Supported | Supported | Supported | |
| Polyline | Supported | Supported | Supported | Supported | Supported | Supported | |
| Arc/Circle | Supported | Supported | Best-effort | Best-effort | Supported | Supported | SVG path近似: IO_CURVE_APPROX_APPLIED |
| Ellipse | Best-effort | Best-effort | Best-effort | Best-effort | Supported | Supported | 分割近似: IO_CURVE_APPROX_APPLIED |
| Spline | Best-effort | Best-effort | Best-effort | Best-effort | Supported | Supported | 近似ε固定: IO_UNSUPPORTED_ENTITY_DXF_SPLINE |
| Text | Best-effort | Best-effort | Supported | Supported | Supported | Supported | フォント差: IO_TEXT_FALLBACK_FONT |
| Dimension | Not supported | Not supported | Not supported | Not supported | Supported | Supported | 未対応通知: IO_UNSUPPORTED_DIMENSION |
| Hatch | Best-effort | Best-effort | Not supported | Not supported | Supported | Supported | 塗り変換制限: IO_HATCH_SIMPLIFIED |
| Block/Symbol | Best-effort | Best-effort | Not supported | Not supported | Supported | Supported | 展開処理: IO_BLOCK_EXPLODED |
| Image | Not supported | Not supported | Supported | Supported | Best-effort | Best-effort | 埋め込み失敗: IO_IMAGE_REFERENCE_DROPPED |
| Units | Supported | Supported | Best-effort | Best-effort | Supported | Supported | 単位推定: IO_UNITS_ASSUMED_MM |
| Layers | Supported | Supported | Best-effort | Best-effort | Supported | Supported | レイヤ名正規化: IO_LAYER_NAME_NORMALIZED |
| Linetypes | Best-effort | Best-effort | Best-effort | Best-effort | Supported | Supported | 線種フォールバック: IO_LINETYPE_FALLBACK |
