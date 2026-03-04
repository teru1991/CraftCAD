use serde::{Deserialize, Serialize};

use crate::reasons::{SecCode, SecError, SecResult};
use crate::ssot::{RepoRoot, SsotPaths};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitsProfile {
    Default,
    Heavy,
}

impl LimitsProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            LimitsProfile::Default => "default",
            LimitsProfile::Heavy => "heavy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitKind {
    ImportBytes,
    SingleEntryBytes,
    SupportZipBytes,
    StringLen,
}

#[derive(Debug, Clone, Copy)]
pub struct ZipStats {
    pub entries: u64,
    pub total_uncompressed_bytes: u64,
    pub max_entry_bytes: u64,
    pub max_path_depth: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LimitsFile {
    version: u64,
    profiles: LimitsProfiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LimitsProfiles {
    #[serde(rename = "default")]
    default_profile: Limits,
    #[serde(rename = "heavy")]
    heavy_profile: Limits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub max_import_bytes: u64,
    pub max_entities: u64,
    pub max_zip_entries: u64,
    pub max_zip_total_uncompressed_bytes: u64,
    pub max_single_entry_bytes: u64,
    pub max_json_depth: u64,
    pub max_string_len: u64,
    pub max_paths_per_entity: u64,
    pub max_points_per_path: u64,
    pub max_support_zip_bytes: u64,
    pub max_path_depth: u64,
}

impl Limits {
    pub fn load_from_ssot(profile: LimitsProfile) -> SecResult<Self> {
        let root = RepoRoot::discover()?;
        let paths = SsotPaths::from_repo_root(&root);
        let s = std::fs::read_to_string(&paths.limits_json).map_err(|e| {
            SecError::new(
                SecCode::SecSsotNotFound,
                format!("read limits.json failed: {e}"),
            )
        })?;
        let f: LimitsFile = serde_json::from_str(&s).map_err(|e| {
            SecError::new(
                SecCode::SecSsotInvalid,
                format!("parse limits.json failed: {e}"),
            )
        })?;
        if f.version < 1 {
            return Err(SecError::new(
                SecCode::SecSsotInvalid,
                "limits.json version must be >= 1",
            ));
        }
        let lim = match profile {
            LimitsProfile::Default => f.profiles.default_profile,
            LimitsProfile::Heavy => f.profiles.heavy_profile,
        };
        lim.validate_basic()?;
        Ok(lim)
    }

    fn validate_basic(&self) -> SecResult<()> {
        // must be >=1
        let all = [
            self.max_import_bytes,
            self.max_entities,
            self.max_zip_entries,
            self.max_zip_total_uncompressed_bytes,
            self.max_single_entry_bytes,
            self.max_json_depth,
            self.max_string_len,
            self.max_paths_per_entity,
            self.max_points_per_path,
            self.max_support_zip_bytes,
            self.max_path_depth,
        ];
        if all.iter().any(|v| *v < 1) {
            return Err(SecError::new(
                SecCode::SecSsotInvalid,
                "limits.json contains value < 1",
            ));
        }
        if self.max_path_depth > 256 {
            return Err(SecError::new(
                SecCode::SecSsotInvalid,
                "max_path_depth too large",
            ));
        }
        Ok(())
    }

    pub fn check_bytes(&self, kind: LimitKind, size: u64) -> SecResult<()> {
        let (limit, label) = match kind {
            LimitKind::ImportBytes => (self.max_import_bytes, "max_import_bytes"),
            LimitKind::SingleEntryBytes => (self.max_single_entry_bytes, "max_single_entry_bytes"),
            LimitKind::SupportZipBytes => (self.max_support_zip_bytes, "max_support_zip_bytes"),
            LimitKind::StringLen => (self.max_string_len, "max_string_len"),
        };
        if size > limit {
            return Err(SecError::new(
                SecCode::SecLimitExceeded,
                format!("{label} exceeded: {size} > {limit}"),
            ));
        }
        Ok(())
    }

    pub fn check_zip(&self, stats: ZipStats) -> SecResult<()> {
        if stats.entries > self.max_zip_entries {
            return Err(SecError::new(
                SecCode::SecZipLimitExceeded,
                format!(
                    "max_zip_entries exceeded: {} > {}",
                    stats.entries, self.max_zip_entries
                ),
            ));
        }
        if stats.total_uncompressed_bytes > self.max_zip_total_uncompressed_bytes {
            return Err(SecError::new(
                SecCode::SecZipLimitExceeded,
                format!(
                    "max_zip_total_uncompressed_bytes exceeded: {} > {}",
                    stats.total_uncompressed_bytes, self.max_zip_total_uncompressed_bytes
                ),
            ));
        }
        if stats.max_entry_bytes > self.max_single_entry_bytes {
            return Err(SecError::new(
                SecCode::SecZipLimitExceeded,
                format!(
                    "max_single_entry_bytes exceeded: {} > {}",
                    stats.max_entry_bytes, self.max_single_entry_bytes
                ),
            ));
        }
        if stats.max_path_depth > self.max_path_depth {
            return Err(SecError::new(
                SecCode::SecPathTooDeep,
                format!(
                    "max_path_depth exceeded: {} > {}",
                    stats.max_path_depth, self.max_path_depth
                ),
            ));
        }
        Ok(())
    }

    pub fn check_json_depth(&self, depth: u64) -> SecResult<()> {
        if depth > self.max_json_depth {
            return Err(SecError::new(
                SecCode::SecJsonDepthExceeded,
                format!("max_json_depth exceeded: {depth} > {}", self.max_json_depth),
            ));
        }
        Ok(())
    }
}
