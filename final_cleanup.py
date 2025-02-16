import os
import shutil
from pathlib import Path

BASE_DIR = Path("CraftCAD/TopScreen")

MOVE_RULES = {
    "Services/ProjectHistory.swift": "Dashboard/Services/",
    "Services/SharedData.swift": "Common/",
    "Services/TagManager.swift": "Dashboard/Services/",
    "View/CADFileManagerView.swift": "APIExtensions/View/",
    "View/DIYModeView.swift": "DIYMode/View/",
    "View/LeatherCraftModeView.swift": "LeatherCraftMode/View/",
    "ViewModel/StorageViewModel.swift": "Dashboard/ViewModel/",
    "UserManagement/Services/UserAuth.swift": "Settings/Services/",
    "UserManagement/Services/UserService.swift": "Settings/Services/"
}

LOG_FILE = BASE_DIR / "migration_log.txt"


def move_files():
    """ 指定されたファイルを適切なフォルダに移動 """
    with open(LOG_FILE, "a") as log:  # ログを追記モードで開く
        for src, dest in MOVE_RULES.items():
            src_path = BASE_DIR / src
            dest_path = BASE_DIR / dest / Path(src).name

            if src_path.exists():
                shutil.move(src_path, dest_path)
                log.write(f"Moved: {src_path} -> {dest_path}\n")
                print(f"📂 移動: {src_path} -> {dest_path}")
            else:
                log.write(f"Not Found: {src_path}\n")
                print(f"⚠️ 見つからない: {src_path}")


def main():
    print("🚀 最終整理を開始...")
    move_files()
    print("✅ 完了！`migration_log.txt` を確認してください。")


if __name__ == "__main__":
    main()
