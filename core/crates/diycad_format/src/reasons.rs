#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReasonCode {
    // Security / limits
    SecZipTraversal,
    SecZipAbsolutePath,
    SecZipPathTooLong,
    SecZipPathTooDeep,
    SecZipTooManyEntries,
    SecZipEntryTooLarge,
    SecZipTotalUncompressedTooLarge,
    SecZipInvalidEntryName,
    SecZipBadZip,

    // Manifest / schema
    OpenManifestMissing,
    OpenManifestInvalidJson,
    OpenManifestSchemaInvalid,
    OpenSchemaForwardIncompatibleReadonly,
    OpenSchemaTooOldSuggestMigrate,

    // Document
    OpenDocumentMissing,
    OpenDocumentInvalidJson,
    OpenDocumentSchemaInvalid,
    OpenDocumentLocateHeuristicUsed,

    // Parts / nest jobs
    OpenPartInvalidJson,
    OpenPartSchemaInvalid,
    OpenNestJobInvalidJson,
    OpenNestJobSchemaInvalid,

    // Integrity
    SaveIntegrityManifestMissing,
    SaveIntegrityEntryMissing,
    SaveIntegritySizeMismatch,
    SaveIntegrityShaMismatch,

    // Save
    SaveValidateFailed,
    SaveAtomicTempCreateFailed,
    SaveAtomicWriteFailed,
    SaveAtomicFsyncFailed,
    SaveAtomicRenameFailed,

    // Migration
    MigrateFailed,
    MigrateApplied,

    // General
    IoReadFailed,
    IoWriteFailed,
}

impl ReasonCode {
    pub fn as_str(&self) -> &'static str {
        use ReasonCode::*;
        match self {
            SecZipTraversal => "SEC_ZIP_TRAVERSAL",
            SecZipAbsolutePath => "SEC_ZIP_ABSOLUTE_PATH",
            SecZipPathTooLong => "SEC_ZIP_PATH_TOO_LONG",
            SecZipPathTooDeep => "SEC_ZIP_PATH_TOO_DEEP",
            SecZipTooManyEntries => "SEC_ZIP_TOO_MANY_ENTRIES",
            SecZipEntryTooLarge => "SEC_ZIP_ENTRY_TOO_LARGE",
            SecZipTotalUncompressedTooLarge => "SEC_ZIP_TOTAL_UNCOMPRESSED_TOO_LARGE",
            SecZipInvalidEntryName => "SEC_ZIP_INVALID_ENTRY_NAME",
            SecZipBadZip => "SEC_ZIP_BAD_ZIP",

            OpenManifestMissing => "OPEN_MANIFEST_MISSING",
            OpenManifestInvalidJson => "OPEN_MANIFEST_INVALID_JSON",
            OpenManifestSchemaInvalid => "OPEN_MANIFEST_SCHEMA_INVALID",
            OpenSchemaForwardIncompatibleReadonly => "OPEN_SCHEMA_FORWARD_INCOMPATIBLE_READONLY",
            OpenSchemaTooOldSuggestMigrate => "OPEN_SCHEMA_TOO_OLD_SUGGEST_MIGRATE",

            OpenDocumentMissing => "OPEN_DOCUMENT_MISSING",
            OpenDocumentInvalidJson => "OPEN_DOCUMENT_INVALID_JSON",
            OpenDocumentSchemaInvalid => "OPEN_DOCUMENT_SCHEMA_INVALID",
            OpenDocumentLocateHeuristicUsed => "OPEN_DOCUMENT_LOCATE_HEURISTIC_USED",

            OpenPartInvalidJson => "OPEN_PART_INVALID_JSON",
            OpenPartSchemaInvalid => "OPEN_PART_SCHEMA_INVALID",
            OpenNestJobInvalidJson => "OPEN_NEST_JOB_INVALID_JSON",
            OpenNestJobSchemaInvalid => "OPEN_NEST_JOB_SCHEMA_INVALID",

            SaveIntegrityManifestMissing => "SAVE_INTEGRITY_MANIFEST_MISSING",
            SaveIntegrityEntryMissing => "SAVE_INTEGRITY_ENTRY_MISSING",
            SaveIntegritySizeMismatch => "SAVE_INTEGRITY_SIZE_MISMATCH",
            SaveIntegrityShaMismatch => "SAVE_INTEGRITY_SHA_MISMATCH",

            SaveValidateFailed => "SAVE_VALIDATE_FAILED",
            SaveAtomicTempCreateFailed => "SAVE_ATOMIC_TEMP_CREATE_FAILED",
            SaveAtomicWriteFailed => "SAVE_ATOMIC_WRITE_FAILED",
            SaveAtomicFsyncFailed => "SAVE_ATOMIC_FSYNC_FAILED",
            SaveAtomicRenameFailed => "SAVE_ATOMIC_RENAME_FAILED",

            MigrateFailed => "MIGRATE_FAILED",
            MigrateApplied => "MIGRATE_APPLIED",

            IoReadFailed => "IO_READ_FAILED",
            IoWriteFailed => "IO_WRITE_FAILED",
        }
    }
}
