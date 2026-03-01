use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpAction {
    OpenProject {
        id: String,
    },
    ImportFile {
        format: String,
        hash: String,
    },
    RunWizard {
        wizard_id: String,
        inputs_hash: String,
    },
    RunNest {
        job_id: String,
    },
    Export {
        format: String,
        options_hash: String,
    },
}
