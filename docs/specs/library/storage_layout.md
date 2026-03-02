# Library Storage Layout

## User data dir
- Windows: `%APPDATA%/CraftCAD/`
- macOS: `~/Library/Application Support/CraftCAD/`
- Linux: `~/.local/share/craftcad/`

## Layout
- `presets/`
  - `<preset_id>@<semver>.json`
- `templates/`
  - `<template_id>@<semver>.json`
- `index/`
  - `library_index.v1.json`（再構築可能、破損時削除OK）
- `logs/`（任意）

## Atomic write方針
- tmpファイルへ書き出し → fsync → rename

## 破損ファイル方針
- 破損は読み飛ばし（落とさない）
- ただしCIでは built-in は破損があれば落とす（SSOTだから）
