# Sample Library SSOT (B02-03)
サンプルは “初回成功体験” と “回帰テスト資産” を兼ねる。壊さない運用を契約化する。

## SSOT
<!-- SSOT:BEGIN -->
kind: sample_library
version: 1
samples:
  - id: sample_shelf_project
    title_key: ux.sample.shelf.title
    description_key: ux.sample.shelf.desc
    file: app/samples/sample_shelf_project.diycad
    read_only: true
    tags: [wood, nesting, print]
    limits:
      max_entities: 5000
      max_parts: 200
  - id: sample_leather_pouch
    title_key: ux.sample.pouch.title
    description_key: ux.sample.pouch.desc
    file: app/samples/sample_leather_pouch.diycad
    read_only: true
    tags: [leather, pattern, print]
    limits:
      max_entities: 3000
      max_parts: 80
policy:
  must_be_openable_offline: true
  schema_compat:
    # サンプルは N-2 まで互換検証対象（詳細はB29-02に従う）
    min_supported: "N-2"
  update_rules:
    # サンプル更新は必ず E2E(golden) を更新せず “比較のみ” で通す（生成禁止）
    require_review: true
<!-- SSOT:END -->
