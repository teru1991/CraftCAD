#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MigrateReason {
    RegistryMissingStep,
    SkipNotAllowed,
    InvalidFromVersion,
    InvalidToVersion,
    TransformFailed,
    ValidateInputFailed,
    ValidateOutputFailed,
}

impl MigrateReason {
    pub fn as_str(&self) -> &'static str {
        use MigrateReason::*;
        match self {
            RegistryMissingStep => "MIGRATE_REGISTRY_MISSING_STEP",
            SkipNotAllowed => "MIGRATE_SKIP_NOT_ALLOWED",
            InvalidFromVersion => "MIGRATE_INVALID_FROM_VERSION",
            InvalidToVersion => "MIGRATE_INVALID_TO_VERSION",
            TransformFailed => "MIGRATE_TRANSFORM_FAILED",
            ValidateInputFailed => "MIGRATE_VALIDATE_INPUT_FAILED",
            ValidateOutputFailed => "MIGRATE_VALIDATE_OUTPUT_FAILED",
        }
    }
}
