use directories::ProjectDirs;
use std::path::PathBuf;

const QUALIFIER: &str = "org";
const ORGANIZATION: &str = "CraftCAD";
const APPLICATION: &str = "CraftCAD";

pub fn app_data_dir() -> Option<PathBuf> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.data_local_dir().to_path_buf())
}

pub fn settings_path() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("settings.json"))
}

pub fn logs_dir() -> Option<PathBuf> {
    app_data_dir().map(|dir| dir.join("logs"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_and_logs_are_under_app_data() {
        let app_dir = app_data_dir().expect("app data dir should resolve in test env");
        let settings = settings_path().expect("settings path should resolve");
        let logs = logs_dir().expect("logs dir should resolve");

        assert_eq!(settings, app_dir.join("settings.json"));
        assert_eq!(logs, app_dir.join("logs"));
    }
}
