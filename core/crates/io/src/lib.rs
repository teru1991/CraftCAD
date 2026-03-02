#![forbid(unsafe_code)]

pub mod model;
pub mod normalize;
pub mod options;
pub mod preflight;
pub mod reasons;
pub mod report;

use std::collections::HashMap;

use model::InternalModel;
use options::{ExportOptions, ImportOptions};
use preflight::preflight_bytes;
use reasons::{AppError, AppResult, ReasonCode};
use report::IoReport;

pub trait Importer: Send + Sync {
    fn format_id(&self) -> &'static str;
    fn import_bytes(&self, bytes: &[u8], opts: &ImportOptions) -> AppResult<ImportResult>;
}

pub trait Exporter: Send + Sync {
    fn format_id(&self) -> &'static str;
    fn export_bytes(&self, model: &InternalModel, opts: &ExportOptions) -> AppResult<ExportResult>;
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub model: InternalModel,
    pub warnings: Vec<AppError>,
    pub report: IoReport,
}

#[derive(Debug, Clone)]
pub struct ExportResult {
    pub bytes: Vec<u8>,
    pub warnings: Vec<AppError>,
    pub report: IoReport,
}

#[derive(Default)]
pub struct IoEngine {
    importers: HashMap<&'static str, Box<dyn Importer>>,
    exporters: HashMap<&'static str, Box<dyn Exporter>>,
}

impl IoEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_importer(mut self, imp: Box<dyn Importer>) -> Self {
        let id = imp.format_id();
        self.importers.insert(id, imp);
        self
    }

    pub fn register_exporter(mut self, exp: Box<dyn Exporter>) -> Self {
        let id = exp.format_id();
        self.exporters.insert(id, exp);
        self
    }

    pub fn has_importer(&self, format_id: &str) -> bool {
        self.importers.contains_key(format_id)
    }

    pub fn has_exporter(&self, format_id: &str) -> bool {
        self.exporters.contains_key(format_id)
    }

    pub fn import(
        &self,
        format_id: &str,
        bytes: &[u8],
        opts: &ImportOptions,
    ) -> AppResult<ImportResult> {
        let imp = self.importers.get(format_id).ok_or_else(|| {
            AppError::new(
                ReasonCode::IoFormatNotRegistered,
                format!("importer not registered: {}", format_id),
            )
            .with_context("format_id", format_id)
        })?;

        preflight_bytes(format_id, bytes, opts)?;

        let mut res = imp.import_bytes(bytes, opts)?;

        let mut warnings = Vec::new();
        normalize::normalize_model(&mut res.model, opts, &mut warnings, &mut res.report);
        res.warnings.extend(warnings);

        res.report.format = format_id.to_string();
        res.report.entities_out = res.model.entities.len();
        res.report.texts_out = res.model.texts.len();
        res.report.determinism_tag = opts.determinism_tag();

        Ok(res)
    }

    pub fn export(
        &self,
        format_id: &str,
        model: &InternalModel,
        opts: &ExportOptions,
    ) -> AppResult<ExportResult> {
        let exp = self.exporters.get(format_id).ok_or_else(|| {
            AppError::new(
                ReasonCode::IoFormatNotRegistered,
                format!("exporter not registered: {}", format_id),
            )
            .with_context("format_id", format_id)
        })?;

        let mut working = model.clone();
        let mut report = IoReport::new(format_id);
        let mut warnings = Vec::new();
        normalize::normalize_model(
            &mut working,
            &opts.as_import_like(),
            &mut warnings,
            &mut report,
        );

        let mut out = exp.export_bytes(&working, opts)?;
        out.warnings.extend(warnings);

        out.report.format = format_id.to_string();
        out.report.entities_in = model.entities.len();
        out.report.texts_in = model.texts.len();
        out.report.entities_out = working.entities.len();
        out.report.texts_out = working.texts.len();
        out.report.determinism_tag = opts.determinism_tag();

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::options::*;
    use crate::reasons::*;

    struct DummyImporter;
    impl Importer for DummyImporter {
        fn format_id(&self) -> &'static str {
            "dummy"
        }

        fn import_bytes(&self, bytes: &[u8], _opts: &ImportOptions) -> AppResult<ImportResult> {
            let mut m = InternalModel::new(Units::Mm);
            if bytes == b"nan" {
                let mut p = PathEntity::new("0".into(), StrokeStyle::default());
                p.segments.push(Segment2D::Line {
                    a: Point2D {
                        x: f64::NAN,
                        y: 0.0,
                    },
                    b: Point2D { x: 1.0, y: 1.0 },
                });
                m.entities.push(Entity::Path(p));
            } else {
                let mut p = PathEntity::new("0".into(), StrokeStyle::default());
                p.segments.push(Segment2D::Line {
                    a: Point2D { x: 0.0, y: 0.0 },
                    b: Point2D { x: 1.0, y: 1.0 },
                });
                m.entities.push(Entity::Path(p));
            }
            let mut r = IoReport::new("dummy");
            r.entities_in = 1;
            Ok(ImportResult {
                model: m,
                warnings: vec![],
                report: r,
            })
        }
    }

    struct DummyExporter;
    impl Exporter for DummyExporter {
        fn format_id(&self) -> &'static str {
            "dummy"
        }

        fn export_bytes(
            &self,
            model: &InternalModel,
            _opts: &ExportOptions,
        ) -> AppResult<ExportResult> {
            let mut r = IoReport::new("dummy");
            r.entities_in = model.entities.len();
            Ok(ExportResult {
                bytes: b"ok".to_vec(),
                warnings: vec![],
                report: r,
            })
        }
    }

    #[test]
    fn engine_import_runs_preflight_and_normalize() {
        let eng = IoEngine::new()
            .register_importer(Box::new(DummyImporter))
            .register_exporter(Box::new(DummyExporter));

        let opts = ImportOptions::default_for_tests();
        let res = eng
            .import("dummy", b"nan", &opts)
            .expect("import should succeed");

        assert_eq!(res.model.entities.len(), 0);
        assert!(res
            .warnings
            .iter()
            .any(|w| w.reason == ReasonCode::IoSanitizeNonfinite));
    }

    #[test]
    fn engine_export_runs_normalize() {
        let eng = IoEngine::new()
            .register_importer(Box::new(DummyImporter))
            .register_exporter(Box::new(DummyExporter));

        let mut m = InternalModel::new(Units::Mm);
        let mut p = PathEntity::new("0".into(), StrokeStyle::default());
        p.segments.push(Segment2D::Line {
            a: Point2D { x: 0.0, y: 0.0 },
            b: Point2D { x: 1.0, y: 1.0 },
        });
        m.entities.push(Entity::Path(p));

        let out = eng
            .export("dummy", &m, &ExportOptions::default_for_tests())
            .expect("export should succeed");
        assert_eq!(out.bytes, b"ok".to_vec());
    }
}
