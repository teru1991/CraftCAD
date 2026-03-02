#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecoveryReason {
    AutosaveSkippedNotDirty,
    AutosaveWriteFailed,
    AutosaveRenameFailed,
    AutosaveFsyncFailed,
    AutosavePruneFailed,
    RestoreListReadFailed,
    RestoreOpenFailed,
    RestoreNoGenerations,
}

impl RecoveryReason {
    pub fn as_str(&self) -> &'static str {
        use RecoveryReason::*;
        match self {
            AutosaveSkippedNotDirty => "RECOVERY_AUTOSAVE_SKIPPED_NOT_DIRTY",
            AutosaveWriteFailed => "RECOVERY_AUTOSAVE_WRITE_FAILED",
            AutosaveRenameFailed => "RECOVERY_AUTOSAVE_RENAME_FAILED",
            AutosaveFsyncFailed => "RECOVERY_AUTOSAVE_FSYNC_FAILED",
            AutosavePruneFailed => "RECOVERY_AUTOSAVE_PRUNE_FAILED",
            RestoreListReadFailed => "RECOVERY_RESTORE_LIST_READ_FAILED",
            RestoreOpenFailed => "RECOVERY_RESTORE_OPEN_FAILED",
            RestoreNoGenerations => "RECOVERY_RESTORE_NO_GENERATIONS",
        }
    }
}
