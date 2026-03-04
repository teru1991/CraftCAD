pub mod cli;
pub mod joblog;
pub mod oplog;
pub mod reason_summary;
pub mod reasons;
pub mod repro;
pub mod retention;
pub mod security_iface;
pub mod ssot_fingerprint;
pub mod store;
pub mod support_zip;

pub use joblog::{DeterminismTag, JobContext, JobLog, JobLogBuilder, StepResultKind};
pub use oplog::{ActionResult, OpLog, OpLogBuilder};
pub use reason_summary::{EmptyCatalogLookup, ReasonCatalogLookup, ReasonSummary};
pub use reasons::Severity;
pub use repro::{generate_repro_markdown, ReproArtifacts, ReproText};
pub use retention::RetentionPolicy;
pub use security_iface::{ConsentProvider, DefaultDenyConsent, Limits, Redactor, StubRedactor};
pub use ssot_fingerprint::SsotFingerprint;

pub use store::{CleanupResult, DiagnosticsStore, StoreIndexEntry};
pub use support_zip::{SupportZipBuilder, ZipResult};

#[derive(Clone)]
pub struct SecurityCtx {
    pub limits: security::Limits,
    pub redactor: security::Redactor,
    pub consent_store: security::ConsentStore,
    pub sandbox: security::Sandbox,
}

impl SecurityCtx {
    pub fn load_default() -> Result<Self, std::io::Error> {
        let limits = security::Limits::load_from_ssot(security::LimitsProfile::Default)
            .map_err(|e| std::io::Error::other(e.message.to_string()))?;
        let redactor = security::Redactor::from_ssot(security::RedactorConfig {
            limits_profile: security::LimitsProfile::Default,
        })
        .map_err(|e| std::io::Error::other(e.message.to_string()))?;
        let consent_store = security::ConsentStore::new_default()
            .map_err(|e| std::io::Error::other(e.message.to_string()))?;
        let sandbox = security::Sandbox::new(security::ExternalRefPolicy::Reject);
        Ok(Self {
            limits,
            redactor,
            consent_store,
            sandbox,
        })
    }
}
