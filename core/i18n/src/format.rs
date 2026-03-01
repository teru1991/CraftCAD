#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSystem {
    Mm,
    Inch,
}

pub fn format_unit(value_mm: f64, unit: UnitSystem) -> String {
    match unit {
        UnitSystem::Mm => format!("{value_mm:.2} mm"),
        UnitSystem::Inch => format!("{:.4} in", value_mm / 25.4),
    }
}
