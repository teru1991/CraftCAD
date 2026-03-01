pub mod version {
    pub type SchemaVersion = u32;
}

use version::SchemaVersion;

pub trait MigrationStep {
    fn from(&self) -> SchemaVersion;
    fn to(&self) -> SchemaVersion;
    fn apply_json(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value>;
    fn summary(&self) -> &'static str;
}

pub struct MigrationReport {
    pub from: SchemaVersion,
    pub to: SchemaVersion,
    pub applied_steps: Vec<String>,
}

pub struct Migrator {
    steps: Vec<Box<dyn MigrationStep + Send + Sync>>,
}

impl Migrator {
    pub fn new(mut steps: Vec<Box<dyn MigrationStep + Send + Sync>>) -> Self {
        steps.sort_by_key(|s| (s.from(), s.to()));
        Self { steps }
    }

    pub fn migrate(
        &self,
        mut v: serde_json::Value,
        from: SchemaVersion,
        to: SchemaVersion,
    ) -> anyhow::Result<(serde_json::Value, MigrationReport)> {
        let mut cur = from;
        let mut applied_steps = Vec::new();
        while cur < to {
            let step = self
                .steps
                .iter()
                .find(|s| s.from() == cur)
                .ok_or_else(|| anyhow::anyhow!("No migration step from {cur}"))?;
            v = step.apply_json(v)?;
            applied_steps.push(format!(
                "{} -> {} ({})",
                step.from(),
                step.to(),
                step.summary()
            ));
            cur = step.to();
        }
        Ok((
            v,
            MigrationReport {
                from,
                to,
                applied_steps,
            },
        ))
    }

    pub fn dry_run(
        &self,
        from: SchemaVersion,
        to: SchemaVersion,
    ) -> anyhow::Result<MigrationReport> {
        let mut cur = from;
        let mut applied_steps = Vec::new();
        while cur < to {
            let step = self
                .steps
                .iter()
                .find(|s| s.from() == cur)
                .ok_or_else(|| anyhow::anyhow!("No migration step from {cur}"))?;
            applied_steps.push(format!(
                "{} -> {} ({})",
                step.from(),
                step.to(),
                step.summary()
            ));
            cur = step.to();
        }
        Ok(MigrationReport {
            from,
            to,
            applied_steps,
        })
    }
}
