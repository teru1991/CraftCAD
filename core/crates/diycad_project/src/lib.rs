use diycad_common::{DiycadError, DiycadResult};

pub const DIYCAD_EXTENSION: &str = ".diycad";

pub fn validate_project_path(path: &str) -> DiycadResult<()> {
  if path.ends_with(DIYCAD_EXTENSION) {
    Ok(())
  } else {
    Err(DiycadError::InvalidState(format!(
      "project file must end with {}",
      DIYCAD_EXTENSION
    )))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_diycad_extension() {
    assert!(validate_project_path("sample.diycad").is_ok());
  }
}
