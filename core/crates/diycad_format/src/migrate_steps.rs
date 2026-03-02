#[cfg(feature = "test_latest_2")]
use migration::report::ChangeSet;
#[cfg(feature = "test_latest_2")]
use migration::step::MigrationStep;
#[cfg(feature = "test_latest_2")]
use serde_json::{json, Value};

#[cfg(feature = "test_latest_2")]
pub struct Step1to2;

#[cfg(feature = "test_latest_2")]
impl MigrationStep for Step1to2 {
    fn from_version(&self) -> i64 {
        1
    }

    fn to_version(&self) -> i64 {
        2
    }

    fn transform_manifest(
        &self,
        manifest: &mut Value,
        changes: &mut ChangeSet,
    ) -> anyhow::Result<()> {
        if manifest.get("determinism_tag").is_none() {
            manifest["determinism_tag"] = json!({"rounding_decimals": 6});
            changes.add_added("/determinism_tag");
        }
        manifest["schema_version"] = json!(2);
        changes.add_changed("/schema_version");
        Ok(())
    }

    fn transform_document(
        &self,
        _document: &mut Value,
        _changes: &mut ChangeSet,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn transform_part(&self, _part: &mut Value, _changes: &mut ChangeSet) -> anyhow::Result<()> {
        Ok(())
    }

    fn transform_nest_job(
        &self,
        _nest_job: &mut Value,
        _changes: &mut ChangeSet,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
