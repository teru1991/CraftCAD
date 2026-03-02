use crate::render_ir::RenderIr;

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("pdf rendering not implemented in this build")]
    NotImplemented,
}

pub fn render_pdf_bytes(_ir: &RenderIr) -> Result<Vec<u8>, PdfError> {
    Err(PdfError::NotImplemented)
}
