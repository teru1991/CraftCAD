use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLanguage {
    Ja,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiUnitSystem {
    Metric,
    Imperial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFontSize {
    Small,
    Normal,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiScale(pub u16);

#[derive(Debug, Clone)]
pub struct UiSettings {
    pub language: UiLanguage,
    pub unit_system: UiUnitSystem,
    pub ui_scale: UiScale,
    pub font_size: UiFontSize,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            language: UiLanguage::Ja,
            unit_system: UiUnitSystem::Metric,
            ui_scale: UiScale(100),
            font_size: UiFontSize::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct I18nRuntime {
    pub locale: &'static str,
    pub fallback_locale: &'static str,
}

impl I18nRuntime {
    pub fn from_settings(settings: &UiSettings) -> Self {
        let locale = match settings.language {
            UiLanguage::Ja => "ja-JP",
            UiLanguage::En => "en-US",
        };
        let fallback_locale = match settings.language {
            UiLanguage::Ja => "en-US",
            UiLanguage::En => "ja-JP",
        };
        Self {
            locale,
            fallback_locale,
        }
    }

    pub fn t(&self, key: &str) -> String {
        key.to_string()
    }

    pub fn t_args(&self, key: &str, _args: &BTreeMap<String, String>) -> String {
        key.to_string()
    }

    pub fn format_length_mm(&self, mm: f64, unit_system: UiUnitSystem) -> String {
        match unit_system {
            UiUnitSystem::Metric => format!("{:.1} mm", mm),
            UiUnitSystem::Imperial => format!("{:.2} in", mm / 25.4),
        }
    }
}
