use craftcad_serialize::{
    BBox, Document, NestJob, Part, PartRef, Placement, Reason, ReasonCode, Result,
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct DeterministicRng(u64);
impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e3779b97f4a7c15)
    }
    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        ((self.0.wrapping_mul(0x2545F4914F6CDD1D) >> 32) & 0xffff_ffff) as u32
    }
}

#[derive(Clone, Debug)]
pub struct PartEval {
    pub part_id: Uuid,
    pub width: f64,
    pub height: f64,
    pub area: f64,
    pub allow_rotate: bool,
}

#[derive(Clone, Debug)]
pub struct PlacementRect {
    pub part_id: Uuid,
    pub sheet_instance_index: u32,
    pub x: f64,
    pub y: f64,
    pub rotation_deg: f64,
    pub width: f64,
    pub height: f64,
}
impl PlacementRect {
    pub fn into_placement(self) -> Placement {
        Placement {
            part_id: self.part_id,
            sheet_instance_index: self.sheet_instance_index,
            x: self.x,
            y: self.y,
            rotation_deg: self.rotation_deg,
            bbox: BBox {
                min_x: self.x,
                min_y: self.y,
                max_x: self.x + self.width,
                max_y: self.y + self.height,
            },
        }
    }
}

fn dims(part: &Part) -> Result<(f64, f64)> {
    let pts = &part.outline.outer;
    if pts.len() < 3 {
        return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
    }
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in pts {
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
        }
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    Ok((max_x - min_x, max_y - min_y))
}

pub fn expand_parts(
    job: &NestJob,
    doc: &Document,
    _rng: &mut DeterministicRng,
) -> Result<Vec<PartEval>> {
    let mut out = vec![];
    let mut refs: Vec<PartRef> = job.parts_ref.clone();
    refs.sort_by_key(|r| r.part_id);
    for r in refs {
        let p = doc
            .parts
            .iter()
            .find(|p| p.id == r.part_id)
            .ok_or_else(|| {
                let mut reason = Reason::from_code(ReasonCode::ModelReferenceNotFound);
                reason
                    .debug
                    .insert("part_id".into(), serde_json::json!(r.part_id));
                reason
            })?;
        let qty = r.quantity_override.unwrap_or(p.quantity).max(1);
        let (w, h) = dims(p)?;
        let inflate =
            job.constraints.global_margin + job.constraints.global_kerf + p.margin + p.kerf;
        let ew = (w + 2.0 * inflate).max(0.0);
        let eh = (h + 2.0 * inflate).max(0.0);
        for _ in 0..qty {
            out.push(PartEval {
                part_id: p.id,
                width: ew,
                height: eh,
                area: ew * eh,
                allow_rotate: p.allow_rotate || job.constraints.allow_rotate_default,
            });
        }
    }
    out.sort_by(|a, b| {
        b.area
            .total_cmp(&a.area)
            .then_with(|| b.width.max(b.height).total_cmp(&a.width.max(a.height)))
            .then_with(|| a.part_id.cmp(&b.part_id))
    });
    Ok(out)
}
