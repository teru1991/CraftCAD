use craftcad_io::model::{StrokeStyle, Units};
use craftcad_io_support::MappingRules;

pub fn map_stroke(mr: &MappingRules, mut s: StrokeStyle) -> StrokeStyle {
    s.layer = mr.map_layer(&s.layer);
    s.linetype = mr.map_linetype(&s.linetype);
    s
}

pub fn map_units(mr: &MappingRules, u: Units) -> Units {
    mr.map_units(u)
}
