use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleId(pub String);

#[derive(Debug, Clone)]
pub struct SampleMeta {
    pub id: SampleId,
    pub title_key: &'static str,
    pub description_key: &'static str,
    pub file_path: PathBuf,
    pub read_only: bool,
    pub tags: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct SampleRegistry {
    root: PathBuf,
    items: Vec<SampleMeta>,
}

impl SampleRegistry {
    pub fn new_default() -> Self {
        let root = PathBuf::from("apps/desktop/assets/samples");
        let items = vec![
            SampleMeta {
                id: SampleId("sample_shelf_project".to_string()),
                title_key: "ux.sample.shelf.title",
                description_key: "ux.sample.shelf.desc",
                file_path: root.join("sample_shelf_project.diycad"),
                read_only: true,
                tags: &["wood", "nesting", "print"],
            },
            SampleMeta {
                id: SampleId("sample_leather_pouch".to_string()),
                title_key: "ux.sample.pouch.title",
                description_key: "ux.sample.pouch.desc",
                file_path: root.join("sample_leather_pouch.diycad"),
                read_only: true,
                tags: &["leather", "pattern", "print"],
            },
        ];
        Self { root, items }
    }

    pub fn list_samples(&self) -> Vec<SampleMeta> {
        self.items.clone()
    }

    pub fn find(&self, id: &SampleId) -> Option<SampleMeta> {
        self.items.iter().find(|s| &s.id == id).cloned()
    }

    pub fn samples_root(&self) -> &Path {
        &self.root
    }
}

#[derive(Debug, Clone)]
pub struct OpenSampleRequest {
    pub sample_id: SampleId,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct OpenSampleResult {
    pub opened_path: PathBuf,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub enum OpenSampleError {
    NotFound,
    Io(String),
    Denied(String),
    Unsupported(String),
}

pub trait SampleOpener {
    fn open_sample(&mut self, req: OpenSampleRequest) -> Result<OpenSampleResult, OpenSampleError>;
}

pub struct ExistenceOnlySampleOpener {
    registry: SampleRegistry,
}

impl ExistenceOnlySampleOpener {
    pub fn new(registry: SampleRegistry) -> Self {
        Self { registry }
    }
}

impl SampleOpener for ExistenceOnlySampleOpener {
    fn open_sample(&mut self, req: OpenSampleRequest) -> Result<OpenSampleResult, OpenSampleError> {
        let meta = self
            .registry
            .find(&req.sample_id)
            .ok_or(OpenSampleError::NotFound)?;
        if !meta.file_path.exists() {
            return Err(OpenSampleError::Io(format!(
                "sample file missing: {}",
                meta.file_path.display()
            )));
        }
        let ro = req.read_only || meta.read_only;
        Ok(OpenSampleResult {
            opened_path: meta.file_path,
            read_only: ro,
        })
    }
}
