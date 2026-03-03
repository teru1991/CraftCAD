# Job Queue SSOT (UI Freeze 防止)

## 1. ルール
- 100ms を超える可能性がある処理は必ずジョブ化する
- ジョブは cancel / progress / priority を必須とする
- 決定性のため、同 priority の実行順は submit 順で固定する

## 2. 優先度の意味
- High: UI 応答に直結する（開く/保存の preflight、軽量再計算など）
- Normal: 通常のバックグラウンド（import/export、再構築など）
- Low: 事前計算/プリフェッチ/重い解析（ユーザー操作を邪魔しない）

## 3. キャンセル規約
- cancel は “例外” ではなく正常系の終了として扱う（JOB_CANCELLED）
- cancel チェックは必ずループ境界に置く（最大反復とセット）
