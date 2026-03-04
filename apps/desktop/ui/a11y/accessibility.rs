/// PR4: UI要素へ name/role を橋渡しできる最小API。
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

impl A11yProps {
    pub const fn new(role: A11yRole, i18n_key: &'static str) -> Self {
        Self {
            role,
            name: A11yName { i18n_key },
        }
    }
}
