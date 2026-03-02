use crate::report::{ChangeSet, VersionBump};
use anyhow::Result;
use serde_json::Value;

pub trait MigrationStep: Send + Sync {
    fn from_version(&self) -> i64;
    fn to_version(&self) -> i64;

    fn transform_manifest(&self, manifest: &mut Value, changes: &mut ChangeSet) -> Result<()>;
    fn transform_document(&self, document: &mut Value, changes: &mut ChangeSet) -> Result<()>;
    fn transform_part(&self, part: &mut Value, changes: &mut ChangeSet) -> Result<()>;
    fn transform_nest_job(&self, nest_job: &mut Value, changes: &mut ChangeSet) -> Result<()>;

    fn logical_validate(&self, _manifest: &Value, _document: &Value) -> Result<()> {
        Ok(())
    }

    fn bump(&self) -> VersionBump {
        VersionBump {
            from: self.from_version(),
            to: self.to_version(),
        }
    }
}
