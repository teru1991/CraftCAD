use super::types::*;

#[derive(Debug, Clone)]
pub enum DimensionEditOp {
    SetManualTextPos { x_mm: f64, y_mm: f64 },
    ClearManualTextPos,
    SetOffsetLevel(u32),
    ToggleSide,
    SetTextOverride(Option<String>),
    SetPrecisionOverride(Option<u8>),
}

pub fn apply_edit(hint: &mut PlacementHint, ov: &mut DimensionOverrides, op: DimensionEditOp) {
    match op {
        DimensionEditOp::SetManualTextPos { x_mm, y_mm } => {
            hint.manual_text_pos_mm = Some((x_mm, y_mm))
        }
        DimensionEditOp::ClearManualTextPos => hint.manual_text_pos_mm = None,
        DimensionEditOp::SetOffsetLevel(v) => hint.offset_level = v,
        DimensionEditOp::ToggleSide => {
            hint.side = match hint.side {
                Side::Left => Side::Right,
                Side::Right => Side::Left,
                Side::Top => Side::Bottom,
                Side::Bottom => Side::Top,
                Side::Auto => Side::Left,
            }
        }
        DimensionEditOp::SetTextOverride(v) => ov.text_override = v,
        DimensionEditOp::SetPrecisionOverride(v) => ov.precision_override = v,
    }
}
