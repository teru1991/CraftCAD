use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

pub fn init_logging(log_dir: impl AsRef<Path>) -> io::Result<PathBuf> {
    let dir = log_dir.as_ref();
    fs::create_dir_all(dir)?;

    let log_path = dir.join("diycad.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let _ = LOG_FILE.set(Mutex::new(file));
    let _ = log_info("logging initialized");

    Ok(log_path)
}

pub fn log_info(message: &str) -> io::Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{}] INFO {}\n", now, message);

    print!("{}", line);

    if let Some(file) = LOG_FILE.get() {
        let mut guard = file
            .lock()
            .map_err(|_| io::Error::other("failed to acquire log file lock"))?;
        guard.write_all(line.as_bytes())?;
        guard.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_logging_creates_log_file() {
        let dir = tempdir().expect("tempdir should be created");
        let log_path = init_logging(dir.path()).expect("init_logging should succeed");
        assert!(log_path.exists());
    }
}
