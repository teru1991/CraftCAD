use migration::report::ChangeSet;
use migration::{MigrationStep, Registry};
use serde_json::{json, Value};

struct Step1to2;
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
        if manifest.get("example_added").is_none() {
            manifest["example_added"] = json!(true);
            changes.add_added("/example_added");
        }
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

#[test]
fn registry_enforces_stepwise_and_reports() {
    let reg = Registry::new().register(Box::new(Step1to2));
    let mut manifest = json!({"schema_version":1});
    let mut document = json!({"id":"d","name":"n","unit":"mm","entities":[],"parts_index":[],"nest_jobs_index":[]});
    let mut parts: Vec<Value> = vec![];
    let mut njs: Vec<Value> = vec![];

    let mut vin = |_v: i64, _m: &Value, _d: &Value| Ok(());
    let mut vout = |_v: i64, _m: &Value, _d: &Value| Ok(());

    let rep = reg
        .migrate_stepwise(
            1,
            2,
            &mut manifest,
            &mut document,
            &mut parts,
            &mut njs,
            (0, 0, 0),
            (0, 0, 0),
            false,
            &mut vin,
            &mut vout,
        )
        .unwrap();

    assert_eq!(rep.overall_from, 1);
    assert_eq!(rep.overall_to, 2);
    assert_eq!(rep.steps.len(), 1);
    assert!(rep.steps[0].changes.added.contains("/example_added"));
}
