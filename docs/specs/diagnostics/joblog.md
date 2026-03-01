# JobLog方針
- 操作は構造化情報のみ記録し、生の個人情報テキストを残さない。
- steps には action_id / params_hash / result / reason_codes を残す。
- path情報は匿名化・相対化して保存する。
