#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Ui,
    Render,
    Io,
    Bench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobType {
    IoImport,
    IoExport,
    Nesting,
    RenderRebuild,
}
