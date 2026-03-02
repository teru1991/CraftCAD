#![forbid(unsafe_code)]

pub mod approx;
pub mod model;
pub mod normalize;
pub mod options;
pub mod postprocess;
pub mod preflight;
pub mod reasons;
pub mod report;

use std::collections::HashMap;

use approx::apply_approx;
use model::InternalModel;
use normalize::normalize_model;
use options::{ExportOptions, ImportOptions};
use postprocess::{
    apply_origin_policy, dedupe_paths, join_paths, optimize_path_order, remove_tiny_segments,
};
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
                ReasonCode::IO_FORMAT_NOT_REGISTERED,
                format!("importer not registered: {}", format_id),
            )
            .with_context("format_id", format_id)
        })?;

        preflight_bytes(format_id, bytes, opts)?;

        let mut res = imp.import_bytes(bytes, opts)?;

        let mut approx_warnings = Vec::new();
        apply_approx(&mut res.model, opts, &mut approx_warnings, &mut res.report);
        res.warnings.extend(approx_warnings);

        let mut norm_warnings = Vec::new();
        normalize_model(&mut res.model, opts, &mut norm_warnings, &mut res.report);
        res.warnings.extend(norm_warnings);

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
                ReasonCode::IO_FORMAT_NOT_REGISTERED,
                format!("exporter not registered: {}", format_id),
            )
            .with_context("format_id", format_id)
        })?;

        let mut working = model.clone();
        let mut report = IoReport::new(format_id);
        let mut warnings: Vec<AppError> = Vec::new();

        normalize_model(
            &mut working,
            &opts.as_import_like(),
            &mut warnings,
            &mut report,
        );
        apply_approx(
            &mut working,
            &opts.as_import_like(),
            &mut warnings,
            &mut report,
        );

        if opts.postprocess {
            apply_origin_policy(
                &mut working,
                &opts.origin_policy,
                &mut warnings,
                &mut report,
            );
            remove_tiny_segments(
                &mut working,
                &opts.as_import_like(),
                &mut warnings,
                &mut report,
            );
            join_paths(
                &mut working,
                &opts.as_import_like(),
                &mut warnings,
                &mut report,
            );
            dedupe_paths(
                &mut working,
                &opts.as_import_like(),
                &mut warnings,
                &mut report,
            );
            optimize_path_order(
                &mut working,
                &opts.as_import_like(),
                &mut warnings,
                &mut report,
            );
        }

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
    use crate::postprocess::{apply_origin_policy, dedupe_paths};
    use crate::reasons::*;

    struct DummyImporter;
    impl Importer for DummyImporter {
        fn format_id(&self) -> &'static str {
            "dummy"
        }

        fn import_bytes(&self, _bytes: &[u8], _opts: &ImportOptions) -> AppResult<ImportResult> {
            let mut m = InternalModel::new(Units::Mm);
            let mut p = PathEntity::new("p1".into(), StrokeStyle::default());
            p.segments.push(Segment2D::CubicBezier {
                a: Point2D { x: 0.0, y: 0.0 },
                c1: Point2D { x: 1.0, y: 2.0 },
                c2: Point2D { x: 2.0, y: 2.0 },
                b: Point2D { x: 3.0, y: 0.0 },
            });
            m.entities.push(Entity::Path(p));
            Ok(ImportResult {
                model: m,
                warnings: vec![],
                report: IoReport::new("dummy"),
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
    fn approx_is_deterministic() {
        let eng = IoEngine::new()
            .register_importer(Box::new(DummyImporter))
            .register_exporter(Box::new(DummyExporter));

        let mut opts = ImportOptions::default_for_tests();
        opts.enable_approx = true;

        let r1 = eng.import("dummy", b"x", &opts).unwrap();
        let r2 = eng.import("dummy", b"x", &opts).unwrap();

        let s1 = match &r1.model.entities[0] {
            Entity::Path(p) => p.segments.len(),
            _ => 0,
        };
        let s2 = match &r2.model.entities[0] {
            Entity::Path(p) => p.segments.len(),
            _ => 0,
        };
        assert_eq!(s1, s2);

        assert!(r1
            .warnings
            .iter()
            .any(|w| w.reason == ReasonCode::IO_CURVE_APPROX_APPLIED));
    }

    #[test]
    fn postprocess_origin_move_to_zero_sets_flag() {
        let mut m = InternalModel::new(Units::Mm);
        let mut p = PathEntity::new("p2".into(), StrokeStyle::default());
        p.segments.push(Segment2D::Line {
            a: Point2D { x: 10.0, y: 10.0 },
            b: Point2D { x: 11.0, y: 11.0 },
        });
        m.entities.push(Entity::Path(p));

        let mut warnings = vec![];
        let mut report = IoReport::new("dummy");
        apply_origin_policy(
            &mut m,
            &OriginPolicy::MoveToZero,
            &mut warnings,
            &mut report,
        );

        assert!(warnings
            .iter()
            .any(|w| w.reason == ReasonCode::IO_ORIGIN_SHIFTED));
    }

    #[test]
    fn postprocess_join_is_reported() {
        let eng = IoEngine::new().register_exporter(Box::new(DummyExporter));

        let mut m = InternalModel::new(Units::Mm);
        let mut p1 = PathEntity::new("a".into(), StrokeStyle::default());
        p1.segments.push(Segment2D::Line {
            a: Point2D { x: 0.0, y: 0.0 },
            b: Point2D { x: 1.0, y: 0.0 },
        });
        let mut p2 = PathEntity::new("b".into(), StrokeStyle::default());
        p2.segments.push(Segment2D::Line {
            a: Point2D { x: 1.0, y: 0.0 },
            b: Point2D { x: 2.0, y: 0.0 },
        });
        m.entities.push(Entity::Path(p1));
        m.entities.push(Entity::Path(p2));

        let mut eopts = ExportOptions::default_for_tests();
        eopts.postprocess = true;
        eopts.enable_approx = false;
        eopts.determinism.join_eps = 1e-6;

        let out = eng.export("dummy", &m, &eopts).unwrap();
        assert!(out
            .warnings
            .iter()
            .any(|w| w.reason == ReasonCode::IO_PATH_JOIN_APPLIED));
    }

    #[test]
    fn dedupe_removes_duplicate_paths() {
        let mut m = InternalModel::new(Units::Mm);
        let mut p1 = PathEntity::new("a".into(), StrokeStyle::default());
        p1.segments.push(Segment2D::Line {
            a: Point2D { x: 0.0, y: 0.0 },
            b: Point2D { x: 1.0, y: 0.0 },
        });
        let mut p2 = PathEntity::new("dup".into(), StrokeStyle::default());
        p2.segments.push(Segment2D::Line {
            a: Point2D { x: 0.0, y: 0.0 },
            b: Point2D { x: 1.0, y: 0.0 },
        });
        m.entities.push(Entity::Path(p1));
        m.entities.push(Entity::Path(p2));

        let opts = ImportOptions::default_for_tests();
        let mut warnings = vec![];
        let mut report = IoReport::new("dummy");
        dedupe_paths(&mut m, &opts, &mut warnings, &mut report);

        assert_eq!(m.entities.len(), 1);
        assert!(warnings
            .iter()
            .any(|w| w.reason == ReasonCode::IO_DEDUP_REMOVED));
    }
}
