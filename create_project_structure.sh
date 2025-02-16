#!/bin/bash

# CraftCAD のルートディレクトリ
ROOT_DIR="CraftCAD"

# ✅ 必要なフォルダを作成
DIRECTORIES=(
    "$ROOT_DIR/TopScreen/Common/Components/Buttons"
    "$ROOT_DIR/TopScreen/Common/Components/Pickers"
    "$ROOT_DIR/TopScreen/Common/Components/Viewers"
    "$ROOT_DIR/LeatherCraftMode/Views"
    "$ROOT_DIR/LeatherCraftMode/ViewModel"
    "$ROOT_DIR/LeatherCraftMode/Model"
    "$ROOT_DIR/LeatherCraftMode/Controller"
    "$ROOT_DIR/LeatherCraftMode/Services"
    "$ROOT_DIR/DIYMode/Views"
    "$ROOT_DIR/DIYMode/ViewModel"
    "$ROOT_DIR/DIYMode/Model"
    "$ROOT_DIR/DIYMode/Controller"
    "$ROOT_DIR/DIYMode/Services"
    "$ROOT_DIR/ProMode/Views"
    "$ROOT_DIR/ProMode/ViewModel"
    "$ROOT_DIR/ProMode/Model"
    "$ROOT_DIR/ProMode/Controller"
    "$ROOT_DIR/ProMode/Services"
    "$ROOT_DIR/Shared/Export"
    "$ROOT_DIR/Shared/Import"
    "$ROOT_DIR/Shared/Utility"
    "$ROOT_DIR/Testing/UnitTests"
    "$ROOT_DIR/Testing/UITests"
    "$ROOT_DIR/Testing/PerformanceTests"
    "$ROOT_DIR/Documentation"
)

# ディレクトリを作成
for dir in "${DIRECTORIES[@]}"; do
    mkdir -p "$dir"
done

echo "📂 ディレクトリ構造を作成しました。"

# ✅ ファイルの移動・複製・作成
declare -A FILE_ACTIONS=(
    ["MaterialPicker.swift"]="$ROOT_DIR/TopScreen/Common/Components/Pickers/"
    ["DesignViewer.swift"]="$ROOT_DIR/TopScreen/Common/Components/Viewers/"
    ["ModeSelectionView.swift"]="$ROOT_DIR/TopScreen/"
    ["MainAppView.swift"]="$ROOT_DIR/TopScreen/"
    ["LeatherCraftModeView.swift"]="$ROOT_DIR/LeatherCraftMode/Views/"
    ["PatternEditorView.swift"]="$ROOT_DIR/LeatherCraftMode/Views/"
    ["SeamAdjustmentView.swift"]="$ROOT_DIR/LeatherCraftMode/Views/"
    ["LeatherCraftViewModel.swift"]="$ROOT_DIR/LeatherCraftMode/ViewModel/"
    ["Leather.swift"]="$ROOT_DIR/LeatherCraftMode/Model/"
    ["Stitching.swift"]="$ROOT_DIR/LeatherCraftMode/Model/"
    ["LeatherCraftController.swift"]="$ROOT_DIR/LeatherCraftMode/Controller/"
    ["LeatherCraftService.swift"]="$ROOT_DIR/LeatherCraftMode/Services/"
    ["DIYModeView.swift"]="$ROOT_DIR/DIYMode/Views/"
    ["CuttingPlanView.swift"]="$ROOT_DIR/DIYMode/Views/"
    ["DIYViewModel.swift"]="$ROOT_DIR/DIYMode/ViewModel/"
    ["Wood.swift"]="$ROOT_DIR/DIYMode/Model/"
    ["Metal.swift"]="$ROOT_DIR/DIYMode/Model/"
    ["Plastic.swift"]="$ROOT_DIR/DIYMode/Model/"
    ["DIYController.swift"]="$ROOT_DIR/DIYMode/Controller/"
    ["CuttingPlanService.swift"]="$ROOT_DIR/DIYMode/Services/"
    ["ProModeView.swift"]="$ROOT_DIR/ProMode/Views/"
    ["AdvancedModelingView.swift"]="$ROOT_DIR/ProMode/Views/"
    ["ProViewModel.swift"]="$ROOT_DIR/ProMode/ViewModel/"
    ["AIRecommendations.swift"]="$ROOT_DIR/ProMode/Model/"
    ["ProModeController.swift"]="$ROOT_DIR/ProMode/Controller/"
    ["AIModelingService.swift"]="$ROOT_DIR/ProMode/Services/"
    ["ExportView.swift"]="$ROOT_DIR/Shared/Export/"
    ["ExportViewModel.swift"]="$ROOT_DIR/Shared/Export/"
    ["ExportHistoryView.swift"]="$ROOT_DIR/Shared/Export/"
    ["ExportHistoryDetailView.swift"]="$ROOT_DIR/Shared/Export/"
    ["ExportService.swift"]="$ROOT_DIR/Shared/Export/"
    ["ImportService.swift"]="$ROOT_DIR/Shared/Import/"
    ["Constants.swift"]="$ROOT_DIR/Shared/Utility/"
    ["Extensions.swift"]="$ROOT_DIR/Shared/Utility/"
    ["FileUtils.swift"]="$ROOT_DIR/Shared/Utility/"
    ["Formatters.swift"]="$ROOT_DIR/Shared/Utility/"
    ["ModelTests.swift"]="$ROOT_DIR/Testing/UnitTests/"
    ["ServiceTests.swift"]="$ROOT_DIR/Testing/UnitTests/"
    ["LeatherCraftUITests.swift"]="$ROOT_DIR/Testing/UITests/"
    ["DIYUITests.swift"]="$ROOT_DIR/Testing/UITests/"
    ["ProUITests.swift"]="$ROOT_DIR/Testing/UITests/"
    ["CuttingPlanPerformanceTests.swift"]="$ROOT_DIR/Testing/PerformanceTests/"
    ["RenderingPerformanceTests.swift"]="$ROOT_DIR/Testing/PerformanceTests/"
    ["API.md"]="$ROOT_DIR/Documentation/"
    ["Architecture.md"]="$ROOT_DIR/Documentation/"
    ["UserGuide.md"]="$ROOT_DIR/Documentation/"
)

# ✅ ファイルの移動またはコピー
for file in "${!FILE_ACTIONS[@]}"; do
    src_path="$file"
    dest_path="${FILE_ACTIONS[$file]}"
    
    if [ -f "$src_path" ]; then
        # ファイルを移動
        mv "$src_path" "$dest_path"
        echo "✅ $file を移動しました → $dest_path"
    else
        # ファイルが存在しない場合は新規作成
        touch "$dest_path/$file"
        echo "📄 $file が見つかりませんでした → 空ファイルを作成"
    fi
done

echo "🎯 ファイルの移動・作成が完了しました。"
