/// PR3では「器のみ」。PR4でUI要素に name/role を付与する実装を行う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A11yRole {
  Button,
  TextField,
  List,
  MenuItem,
  Dialog,
  Canvas,
}

#[derive(Debug, Clone)]
pub struct A11yName {
  /// i18n key（値直書き禁止）
  pub i18n_key: &'static str,
}

#[derive(Debug, Clone)]
pub struct A11yProps {
  pub role: A11yRole,
  pub name: A11yName,
}
