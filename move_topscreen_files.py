import os
import shutil
from pathlib import Path

# TopScreen の正しい場所を設定
BASE_DIR = Path("CraftCAD/TopScreen")

# 各機能フォルダの構成
FEATURE_FOLDERS = {
    "ModeSwitch": {
        "Controller": ["ModeManager.swift", "ModeSelector.swift", "UserController.swift"],
        "View": ["ModeSelectorView.swift", "ModeIcon.swift", "ModeHighlight.swift"],
        "ViewModel": ["UserViewModel.swift"],
        "Model": ["User.swift"],
        "Services": ["LanguageSelector.swift", "ThemeManager.swift", "ResetSettings.swift", "ShortcutSettingsView.swift"]
    },
    "Dashboard": {
        "Controller": ["ProjectManager.swift"],
        "View": ["DashboardView.swift", "UserInfoPanel.swift", "StorageStatus.swift", "ProjectListView.swift", "ProjectList.swift"],
        "ViewModel": ["DashboardViewModel.swift", "ProjectViewModel.swift"],
        "Model": ["Project.swift", "Storage.swift"],
        "Services": ["ProjectService.swift", "StorageService.swift", "LogManager.swift", "ErrorHandler.swift"]
    },
    "CloudSync": {
        "Controller": ["SyncManager.swift"],
        "View": ["ExportHistoryView.swift", "ExportHistoryDetailView.swift"],
        "Model": ["SyncHistory.swift"],
        "Services": ["CloudStorageAdapter.swift", "CloudSyncService.swift"]
    },
    "APIExtensions": {
        "Controller": [],
        "View": ["PluginManagerView.swift", "ScriptExecutorView.swift"],
        "Model": [],
        "Services": ["APIService.swift", "CADIntegration.swift", "PluginManager.swift", "ScriptExecutor.swift", "ConfigManager.swift"]
    },
    "Settings": {
        "Controller": ["SettingsController.swift"],
        "View": ["SettingsView.swift"],
        "Model": ["SettingsModel.swift"],
        "Services": ["ThemeManager.swift", "LanguageSelector.swift", "ShortcutSettingsView.swift", "NotificationSettings.swift"]
    }
}

# ログファイル
LOG_FILE = BASE_DIR / "migration_log.txt"


def create_directories():
    """ 各機能フォルダとそのサブフォルダを作成 """
    for feature, subfolders in FEATURE_FOLDERS.items():
        feature_path = BASE_DIR / feature
        for subfolder in subfolders.keys():
            (feature_path / subfolder).mkdir(parents=True, exist_ok=True)
    print("✅ フォルダ作成完了！")


def find_file(filename):
    """ `TopScreen/` 以下のすべてのサブフォルダを検索し、ファイルの絶対パスを取得 """
    for root, _, files in os.walk(BASE_DIR):
        if filename in files:
            return os.path.join(root, filename)
    return None


def move_files():
    """ 各ファイルを適切なサブフォルダに移動 """
    with open(LOG_FILE, "w") as log:
        for feature, subfolders in FEATURE_FOLDERS.items():
            for subfolder, files in subfolders.items():
                subfolder_path = BASE_DIR / feature / subfolder
                for file_name in files:
                    src_path = find_file(file_name)  # ファイルを検索
                    if src_path:
                        dst_path = subfolder_path / file_name
                        shutil.move(src_path, dst_path)
                        log.write(f"Moved: {src_path} -> {dst_path}\n")
                        print(f"📂 移動: {src_path} -> {dst_path}")
                    else:
                        log.write(f"Not Found: {file_name}\n")
                        print(f"⚠️ 見つからない: {file_name}")


def clean_empty_dirs():
    """ 空フォルダを削除 """
    for dirpath, dirnames, filenames in os.walk(BASE_DIR, topdown=False):
        if dirpath == str(BASE_DIR):
            continue
        if not dirnames and not filenames:
            os.rmdir(dirpath)
            print(f"🗑 削除: {dirpath}")


def main():
    print("🚀 ファイル整理を開始...")
    create_directories()
    move_files()
    clean_empty_dirs()
    print("✅ 完了！`migration_log.txt` を確認してください。")


if __name__ == "__main__":
    main()
