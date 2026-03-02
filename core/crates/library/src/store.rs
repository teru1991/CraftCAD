use crate::index::LibraryIndex;
use crate::reasons::{LibraryReason, LibraryReasonCode};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LibraryLayout {
    pub root: PathBuf,
}

impl LibraryLayout {
    pub fn index_dir(&self) -> PathBuf {
        self.root.join("index")
    }

    pub fn index_file(&self) -> PathBuf {
        self.index_dir().join("library_index.v1.json")
    }

    pub fn ensure_dirs(&self) -> Result<(), LibraryReason> {
        fs::create_dir_all(self.index_dir()).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibIoError,
                format!("create_dir_all failed: {e}"),
            )
            .with_path(self.index_dir().display().to_string())
        })
    }
}

pub fn save_index_atomic(
    layout: &LibraryLayout,
    idx: &LibraryIndex,
) -> Result<PathBuf, LibraryReason> {
    layout.ensure_dirs()?;
    let dest = layout.index_file();
    let parent = dest.parent().expect("index file must have parent");

    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(|e| {
        LibraryReason::new(
            LibraryReasonCode::LibIoError,
            format!("tempfile create failed: {e}"),
        )
        .with_path(parent.display().to_string())
    })?;

    let s = serde_json::to_string_pretty(idx).map_err(|e| {
        LibraryReason::new(
            LibraryReasonCode::LibIoError,
            format!("serialize failed: {e}"),
        )
        .with_path(dest.display().to_string())
    })?;

    use std::io::Write;
    tmp.write_all(s.as_bytes()).map_err(|e| {
        LibraryReason::new(LibraryReasonCode::LibIoError, format!("write failed: {e}"))
            .with_path(dest.display().to_string())
    })?;
    tmp.flush().map_err(|e| {
        LibraryReason::new(LibraryReasonCode::LibIoError, format!("flush failed: {e}"))
            .with_path(dest.display().to_string())
    })?;

    if dest.exists() {
        fs::remove_file(&dest).map_err(|e| {
            LibraryReason::new(
                LibraryReasonCode::LibIoError,
                format!("remove existing failed: {e}"),
            )
            .with_path(dest.display().to_string())
        })?;
    }

    tmp.persist(&dest).map_err(|e| {
        LibraryReason::new(
            LibraryReasonCode::LibIoError,
            format!("persist failed: {e}"),
        )
        .with_path(dest.display().to_string())
    })?;

    Ok(dest)
}

pub fn load_index_or_rebuild(
    layout: &LibraryLayout,
    rebuild: impl FnOnce() -> Result<LibraryIndex, LibraryReason>,
) -> Result<(LibraryIndex, Vec<LibraryReason>), LibraryReason> {
    layout.ensure_dirs()?;
    let mut warnings = vec![];
    let p = layout.index_file();

    if p.exists() {
        match fs::read_to_string(&p) {
            Ok(s) => match serde_json::from_str::<LibraryIndex>(&s) {
                Ok(idx) => return Ok((idx, warnings)),
                Err(e) => warnings.push(
                    LibraryReason::new(
                        LibraryReasonCode::LibIndexCorrupt,
                        format!("index parse failed: {e}"),
                    )
                    .with_path(p.display().to_string()),
                ),
            },
            Err(e) => warnings.push(
                LibraryReason::new(
                    LibraryReasonCode::LibIoError,
                    format!("index read failed: {e}"),
                )
                .with_path(p.display().to_string()),
            ),
        }
    }

    let idx = rebuild()?;
    warnings.push(LibraryReason::new(
        LibraryReasonCode::LibIndexRebuilt,
        "index rebuilt",
    ));
    let _ = save_index_atomic(layout, &idx);

    Ok((idx, warnings))
}
