pub mod actions;
pub mod error_mapper;
pub mod error_panel;
pub mod history;
pub mod reason_catalog;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub reason_code: String,
    pub severity: Severity,
    pub context: BTreeMap<String, String>,
    pub job_id: Option<String>,
    pub op_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionKind {
    OpenDocs,
    OpenSettings,
    CreateSupportZip,
    RunMigrateTool,
    RetryLastJob,
    JumpToEntity,
    DuplicateSampleAsProject,
    CancelActiveJob,
    ShowJobProgress,
}

impl ActionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionKind::OpenDocs => "OpenDocs",
            ActionKind::OpenSettings => "OpenSettings",
            ActionKind::CreateSupportZip => "CreateSupportZip",
            ActionKind::RunMigrateTool => "RunMigrateTool",
            ActionKind::RetryLastJob => "RetryLastJob",
            ActionKind::JumpToEntity => "JumpToEntity",
            ActionKind::DuplicateSampleAsProject => "DuplicateSampleAsProject",
            ActionKind::CancelActiveJob => "CancelActiveJob",
            ActionKind::ShowJobProgress => "ShowJobProgress",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFacingAction {
    pub kind: ActionKind,
    pub label_key: String,
    pub args: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFacingError {
    pub title_key: String,
    pub detail_key: String,
    pub why_key: Option<String>,
    pub actions: Vec<UserFacingAction>,
    pub doc_link: Option<String>,
    pub debug_ref: DebugRef,
    pub display_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugRef {
    pub reason_code: String,
    pub severity: Severity,
    pub job_id: Option<String>,
    pub op_id: Option<String>,
    pub context: BTreeMap<String, String>,
}

use error_mapper::ErrorMapper;
use error_panel::ErrorPanelView;
use history::ErrorHistory;
use reason_catalog::{CatalogPaths, ReasonCatalog};

pub struct ErrorUxController {
    catalog: ReasonCatalog,
    mapper: ErrorMapper,
    history: ErrorHistory,
    current: Option<UserFacingError>,
    visible: bool,
}

impl ErrorUxController {
    pub fn new() -> Result<Self, String> {
        let catalog = ReasonCatalog::load(&CatalogPaths::default())?;
        Ok(Self {
            catalog,
            mapper: ErrorMapper::default(),
            history: ErrorHistory::new(20),
            current: None,
            visible: false,
        })
    }

    pub fn show(&mut self, e: AppError) {
        let u = self.mapper.map(&e, &self.catalog);
        self.history.push(u.clone());
        self.current = Some(u);
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn view(&self) -> Option<ErrorPanelView> {
        self.current
            .as_ref()
            .map(|e| ErrorPanelView::from_error(e, self.visible))
    }

    pub fn recent(&self) -> Vec<(String, String)> {
        self.history.list()
    }

    pub fn reopen(&mut self, hash: &str) -> bool {
        if let Some(e) = self.history.find_by_hash(hash) {
            self.current = Some(e);
            self.visible = true;
            true
        } else {
            false
        }
    }

    pub fn click_action(
        &self,
        idx: usize,
    ) -> Result<actions::ActionEffect, actions::ActionExecError> {
        let Some(e) = &self.current else {
            return Ok(actions::ActionEffect::None);
        };
        let Some(a) = e.actions.get(idx) else {
            return Ok(actions::ActionEffect::None);
        };
        actions::ActionExecutor::exec(a)
    }
}
