#!/bin/bash

# CraftCAD のベースディレクトリ（実行する前に適宜変更してください）
BASE_DIR="CraftCAD"

# モードのディレクトリを作成
mkdir -p "$BASE_DIR/TopScreen"
mkdir -p "$BASE_DIR/DIYMode"
mkdir -p "$BASE_DIR/LeatherCraftMode"
mkdir -p "$BASE_DIR/ProMode"
mkdir -p "$BASE_DIR/Shared/Utility"

# ファイル移動関数（ディレクトリがない場合は作成）
move_file() {
    src="$BASE_DIR/$1"
    dest="$BASE_DIR/$2"
    
    if [ -f "$src" ]; then
        mkdir -p "$(dirname "$dest")"
        mv "$src" "$dest"
        echo "Moved: $src -> $dest"
    fi
}

# シンボリックリンク作成関数
link_file() {
    src="$BASE_DIR/$1"
    dest="$BASE_DIR/$2"
    
    if [ -f "$src" ]; then
        ln -s "$src" "$dest"
        echo "Linked: $dest -> $src"
    fi
}

# === TopScreen 用ファイルの移動 ===
move_file "Controller/UserController.swift" "TopScreen/Controller/UserController.swift"
move_file "Services/UserService.swift" "TopScreen/Services/UserService.swift"
move_file "Model/User.swift" "TopScreen/Model/User.swift"
move_file "View/SettingsView.swift" "TopScreen/View/SettingsView.swift"
move_file "View/MainView.swift" "TopScreen/View/MainView.swift"
move_file "View/ExportHistoryView.swift" "TopScreen/View/ExportHistoryView.swift"
move_file "View/ExportHistoryDetailView.swift" "TopScreen/View/ExportHistoryDetailView.swift"

# === DIYMode 用ファイルの移動 ===
move_file "Controller/DesignController.swift" "DIYMode/Controller/DesignController.swift"
move_file "Controller/CuttingPlanController.swift" "DIYMode/Controller/CuttingPlanController.swift"
move_file "Model/CuttingPlan.swift" "DIYMode/Model/CuttingPlan.swift"
move_file "Model/Materials/Wood.swift" "DIYMode/Model/Materials/Wood.swift"
move_file "Model/Materials/Plastic.swift" "DIYMode/Model/Materials/Plastic.swift"
move_file "Model/Designs/2DDesign.swift" "DIYMode/Model/Designs/2DDesign.swift"
move_file "Model/Designs/3DDesign.swift" "DIYMode/Model/Designs/3DDesign.swift"
move_file "Services/CuttingPlanService.swift" "DIYMode/Services/CuttingPlanService.swift"
move_file "Services/DesignService.swift" "DIYMode/Services/DesignService.swift"
move_file "View/CuttingPlanView.swift" "DIYMode/View/CuttingPlanView.swift"
move_file "View/TwoDPatternView.swift" "DIYMode/View/TwoDPatternView.swift"
move_file "ViewModel/DesignViewModel.swift" "DIYMode/ViewModel/DesignViewModel.swift"
move_file "ViewModel/CuttingPlanViewModel.swift" "DIYMode/ViewModel/CuttingPlanViewModel.swift"

# === LeatherCraftMode 用ファイルの移動 ===
move_file "Model/Materials/Leather.swift" "LeatherCraftMode/Model/Materials/Leather.swift"
move_file "Controller/ExportController.swift" "LeatherCraftMode/Controller/ExportController.swift"
move_file "Model/Paint.swift" "LeatherCraftMode/Model/Paint.swift"
move_file "Services/ExportService.swift" "LeatherCraftMode/Services/ExportService.swift"
move_file "View/ExportView.swift" "LeatherCraftMode/View/ExportView.swift"
move_file "View/Dialogs/ExportSettingsDialog.swift" "LeatherCraftMode/View/Dialogs/ExportSettingsDialog.swift"
move_file "View/Modes/PaintModeView.swift" "LeatherCraftMode/View/Modes/PaintModeView.swift"
move_file "ViewModel/ExportViewModel.swift" "LeatherCraftMode/ViewModel/ExportViewModel.swift"

# === ProMode 用ファイルの移動 ===
move_file "Controller/SimulationController.swift" "ProMode/Controller/SimulationController.swift"
move_file "Model/Analysis/AIRecommendations.swift" "ProMode/Model/Analysis/AIRecommendations.swift"
move_file "Model/Analysis/Sustainability.swift" "ProMode/Model/Analysis/Sustainability.swift"
move_file "Model/History/DesignHistory.swift" "ProMode/Model/History/DesignHistory.swift"
move_file "Model/History/VersionControl.swift" "ProMode/Model/History/VersionControl.swift"
move_file "Model/Collaboration/RealTimeCollaboration.swift" "ProMode/Model/Collaboration/RealTimeCollaboration.swift"
move_file "Model/Collaboration/CollaborationSettings.swift" "ProMode/Model/Collaboration/CollaborationSettings.swift"
move_file "Services/SimulationService.swift" "ProMode/Services/SimulationService.swift"
move_file "Services/HistoryService.swift" "ProMode/Services/HistoryService.swift"
move_file "Services/CollaborationService.swift" "ProMode/Services/CollaborationService.swift"
move_file "View/SimulationView.swift" "ProMode/View/SimulationView.swift"
move_file "View/CollaborationView.swift" "ProMode/View/CollaborationView.swift"
move_file "ViewModel/SimulationViewModel.swift" "ProMode/ViewModel/SimulationViewModel.swift"
move_file "ViewModel/CollaborationViewModel.swift" "ProMode/ViewModel/CollaborationViewModel.swift"

# シンボリックリンクを作成（共通ファイル）
link_file "Shared/Utility/GeometryUtils.swift" "DIYMode/Utility/GeometryUtils.swift"
link_file "Shared/Utility/GeometryUtils.swift" "LeatherCraftMode/Utility/GeometryUtils.swift"
link_file "Shared/Utility/GeometryUtils.swift" "ProMode/Utility/GeometryUtils.swift"

echo "整理完了！"
