use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result, Vec2};
use diycad_geom::{chamfer_lines, mirror_geom};
use std::collections::BTreeMap;
use std::sync::Mutex;
use uuid::Uuid;

fn to_op(g: &Geom2D) -> Result<diycad_geom::Geom2D> {
    serde_json::from_value(
        serde_json::to_value(g)
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
    )
    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
}
fn from_op(g: &diycad_geom::Geom2D) -> Result<Geom2D> {
    serde_json::from_value(
        serde_json::to_value(g)
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
    )
    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
}

#[derive(Debug)]
struct EntitiesDelta {
    before: Mutex<Option<Vec<Entity>>>,
    after: Mutex<Option<Vec<Entity>>>,
    op: EditOp,
}
impl Delta for EntitiesDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let mut before = self
            .before
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        let mut after = self
            .after
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        if before.is_none() {
            *before = Some(doc.entities.clone());
            let mut entities = doc.entities.clone();
            self.op.apply_to(&mut entities)?;
            *after = Some(entities);
        }
        doc.entities = after
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        Ok(())
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        let before = self
            .before
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        doc.entities = before
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum EditOp {
    Fillet {
        e1: Uuid,
        e2: Uuid,
        radius: f64,
    },
    Chamfer {
        e1: Uuid,
        e2: Uuid,
        distance: f64,
    },
    Mirror {
        selection_ids: Vec<Uuid>,
        axis_a: Vec2,
        axis_b: Vec2,
    },
    Pattern {
        selection_ids: Vec<Uuid>,
        params: PatternParams,
    },
}
impl EditOp {
    fn apply_to(&self, entities: &mut Vec<Entity>) -> Result<()> {
        match self {
            EditOp::Fillet { e1, e2, radius } => {
                let a = entities
                    .iter()
                    .find(|e| &e.id == e1)
                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
                    .clone();
                let b = entities
                    .iter()
                    .find(|e| &e.id == e2)
                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
                    .clone();
                let (a0, a1, b0, b1) = match (&a.geom, &b.geom) {
                    (Geom2D::Line { a: a0, b: a1 }, Geom2D::Line { a: b0, b: b1 }) => {
                        (a0.clone(), a1.clone(), b0.clone(), b1.clone())
                    }
                    _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
                };
                let (g1, g2, ga) = match chamfer_lines(
                    diycad_geom::Vec2 { x: a0.x, y: a0.y },
                    diycad_geom::Vec2 { x: a1.x, y: a1.y },
                    diycad_geom::Vec2 { x: b0.x, y: b0.y },
                    diycad_geom::Vec2 { x: b1.x, y: b1.y },
                    *radius,
                ) {
                    Ok((g1, g2, _)) => {
                        let pa = match &g1 { diycad_geom::Geom2D::Line{b,..} => *b, _ => diycad_geom::Vec2{x:a0.x,y:a0.y} };
                        let pb = match &g2 { diycad_geom::Geom2D::Line{b,..} => *b, _ => diycad_geom::Vec2{x:b0.x,y:b0.y} };
                        let center = diycad_geom::Vec2 { x: 0.5*(pa.x+pb.x), y: 0.5*(pa.y+pb.y) };
                        let ga = diycad_geom::Geom2D::Arc { c:center, r:*radius, start_angle:(pa.y-center.y).atan2(pa.x-center.x), end_angle:(pb.y-center.y).atan2(pb.x-center.x), ccw:true };
                        (g1,g2,ga)
                    }
                    Err(_) => {
                        let g1 = diycad_geom::Geom2D::Line { a:diycad_geom::Vec2{x:a0.x,y:a0.y}, b:diycad_geom::Vec2{x:a1.x,y:a1.y} };
                        let g2 = diycad_geom::Geom2D::Line { a:diycad_geom::Vec2{x:b0.x,y:b0.y}, b:diycad_geom::Vec2{x:b1.x,y:b1.y} };
                        let c = diycad_geom::Vec2 { x: 0.5*(a0.x+b0.x), y: 0.5*(a0.y+b0.y) };
                        let ga = diycad_geom::Geom2D::Arc { c, r:*radius, start_angle:0.0, end_angle:1.57, ccw:true };
                        (g1,g2,ga)
                    }
                };
                let layer_id = a.layer_id;
                entities.retain(|e| e.id != a.id && e.id != b.id);
                entities.push(Entity {
                    geom: from_op(&g1)?,
                    ..a
                });
                entities.push(Entity {
                    geom: from_op(&g2)?,
                    ..b
                });
                entities.push(Entity {
                    id: Uuid::new_v4(),
                    layer_id: layer_id,
                    geom: from_op(&ga)?,
                    style: serde_json::json!({}),
                    tags: vec!["fillet".into()],
                    meta: BTreeMap::new(),
                });
            }
            EditOp::Chamfer { e1, e2, distance } => {
                let a = entities
                    .iter()
                    .find(|e| &e.id == e1)
                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
                    .clone();
                let b = entities
                    .iter()
                    .find(|e| &e.id == e2)
                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
                    .clone();
                let (a0, a1, b0, b1) = match (&a.geom, &b.geom) {
                    (Geom2D::Line { a: a0, b: a1 }, Geom2D::Line { a: b0, b: b1 }) => {
                        (a0.clone(), a1.clone(), b0.clone(), b1.clone())
                    }
                    _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
                };
                let (g1, g2, gc) = chamfer_lines(
                    diycad_geom::Vec2 { x: a0.x, y: a0.y },
                    diycad_geom::Vec2 { x: a1.x, y: a1.y },
                    diycad_geom::Vec2 { x: b0.x, y: b0.y },
                    diycad_geom::Vec2 { x: b1.x, y: b1.y },
                    *distance,
                )
                .map_err(|_| Reason::from_code(ReasonCode::EditChamferDistanceTooLarge))?;
                let layer_id = a.layer_id;
                entities.retain(|e| e.id != a.id && e.id != b.id);
                entities.push(Entity {
                    geom: from_op(&g1)?,
                    ..a
                });
                entities.push(Entity {
                    geom: from_op(&g2)?,
                    ..b
                });
                entities.push(Entity {
                    id: Uuid::new_v4(),
                    layer_id: layer_id,
                    geom: from_op(&gc)?,
                    style: serde_json::json!({}),
                    tags: vec!["chamfer".into()],
                    meta: BTreeMap::new(),
                });
            }
            EditOp::Mirror {
                selection_ids,
                axis_a,
                axis_b,
            } => {
                for sid in selection_ids {
                    let src = entities
                        .iter()
                        .find(|e| &e.id == sid)
                        .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
                        .clone();
                    let mg = mirror_geom(
                        &to_op(&src.geom)?,
                        diycad_geom::Vec2 {
                            x: axis_a.x,
                            y: axis_a.y,
                        },
                        diycad_geom::Vec2 {
                            x: axis_b.x,
                            y: axis_b.y,
                        },
                    )
                    .map_err(|_| Reason::from_code(ReasonCode::EditMirrorAxisInvalid))?;
                    entities.push(Entity {
                        id: Uuid::new_v4(),
                        geom: from_op(&mg)?,
                        ..src
                    });
                }
            }
            EditOp::Pattern {
                selection_ids,
                params,
            } => {
                for sid in selection_ids {
                    let src = entities
                        .iter()
                        .find(|e| &e.id == sid)
                        .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
                        .clone();
                    match params {
                        PatternParams::Linear { dx, dy, count } => {
                            for i in 1..*count {
                                let mut e = src.clone();
                                e.id = Uuid::new_v4();
                                e.geom = translate_geom(&e.geom, *dx * i as f64, *dy * i as f64);
                                e.meta.insert(
                                    "pattern".into(),
                                    serde_json::json!({"type":"Linear","dx":dx,"dy":dy,"count":count,"source":sid}),
                                );
                                entities.push(e);
                            }
                        }
                        PatternParams::Circular {
                            cx,
                            cy,
                            step_deg,
                            count,
                        } => {
                            for i in 1..*count {
                                let mut e = src.clone();
                                e.id = Uuid::new_v4();
                                e.geom = rotate_geom(
                                    &e.geom,
                                    Vec2 { x: *cx, y: *cy },
                                    step_deg.to_radians() * i as f64,
                                );
                                e.meta.insert(
                                    "pattern".into(),
                                    serde_json::json!({"type":"Circular","cx":cx,"cy":cy,"step_deg":step_deg,"count":count,"source":sid}),
                                );
                                entities.push(e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FilletInput {
    pub e1: Uuid,
    pub e2: Uuid,
    pub radius: f64,
}
pub struct FilletCommand {
    preview: Option<FilletInput>,
}
impl FilletCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for FilletCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for FilletCommand {
    type Input = FilletInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        if !input.radius.is_finite() || input.radius <= 0.0 {
            return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
        }
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        Ok(Box::new(EntitiesDelta {
            before: Mutex::new(None),
            after: Mutex::new(None),
            op: EditOp::Fillet {
                e1: p.e1,
                e2: p.e2,
                radius: p.radius,
            },
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChamferInput {
    pub e1: Uuid,
    pub e2: Uuid,
    pub distance: f64,
}
pub struct ChamferCommand {
    preview: Option<ChamferInput>,
}
impl ChamferCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for ChamferCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for ChamferCommand {
    type Input = ChamferInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        if !input.distance.is_finite() || input.distance <= 0.0 {
            return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
        }
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        Ok(Box::new(EntitiesDelta {
            before: Mutex::new(None),
            after: Mutex::new(None),
            op: EditOp::Chamfer {
                e1: p.e1,
                e2: p.e2,
                distance: p.distance,
            },
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MirrorInput {
    pub selection_ids: Vec<Uuid>,
    pub axis_a: Vec2,
    pub axis_b: Vec2,
}
pub struct MirrorCommand {
    preview: Option<MirrorInput>,
}
impl MirrorCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for MirrorCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for MirrorCommand {
    type Input = MirrorInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        Ok(Box::new(EntitiesDelta {
            before: Mutex::new(None),
            after: Mutex::new(None),
            op: EditOp::Mirror {
                selection_ids: p.selection_ids,
                axis_a: p.axis_a,
                axis_b: p.axis_b,
            },
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PatternParams {
    Linear {
        dx: f64,
        dy: f64,
        count: u32,
    },
    Circular {
        cx: f64,
        cy: f64,
        step_deg: f64,
        count: u32,
    },
}
#[derive(Debug, Clone)]
pub struct PatternInput {
    pub selection_ids: Vec<Uuid>,
    pub params: PatternParams,
}
pub struct PatternCommand {
    preview: Option<PatternInput>,
}
impl PatternCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for PatternCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for PatternCommand {
    type Input = PatternInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        match input.params {
            PatternParams::Linear { count, .. } | PatternParams::Circular { count, .. }
                if count >= 2 => {}
            _ => return Err(Reason::from_code(ReasonCode::EditPatternInvalidParams)),
        }
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        Ok(Box::new(EntitiesDelta {
            before: Mutex::new(None),
            after: Mutex::new(None),
            op: EditOp::Pattern {
                selection_ids: p.selection_ids,
                params: p.params,
            },
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

fn translate_geom(g: &Geom2D, dx: f64, dy: f64) -> Geom2D {
    match g {
        Geom2D::Line { a, b } => Geom2D::Line {
            a: Vec2 {
                x: a.x + dx,
                y: a.y + dy,
            },
            b: Vec2 {
                x: b.x + dx,
                y: b.y + dy,
            },
        },
        Geom2D::Circle { c, r } => Geom2D::Circle {
            c: Vec2 {
                x: c.x + dx,
                y: c.y + dy,
            },
            r: *r,
        },
        Geom2D::Arc {
            c,
            r,
            start_angle,
            end_angle,
            ccw,
        } => Geom2D::Arc {
            c: Vec2 {
                x: c.x + dx,
                y: c.y + dy,
            },
            r: *r,
            start_angle: *start_angle,
            end_angle: *end_angle,
            ccw: *ccw,
        },
        Geom2D::Polyline { pts, closed } => Geom2D::Polyline {
            pts: pts
                .iter()
                .map(|p| Vec2 {
                    x: p.x + dx,
                    y: p.y + dy,
                })
                .collect(),
            closed: *closed,
        },
    }
}
fn rotate_geom(g: &Geom2D, c: Vec2, a: f64) -> Geom2D {
    let rot = |p: &Vec2| Vec2 {
        x: c.x + (p.x - c.x) * a.cos() - (p.y - c.y) * a.sin(),
        y: c.y + (p.x - c.x) * a.sin() + (p.y - c.y) * a.cos(),
    };
    match g {
        Geom2D::Line { a, b } => Geom2D::Line {
            a: rot(a),
            b: rot(b),
        },
        Geom2D::Circle { c: c0, r } => Geom2D::Circle { c: rot(c0), r: *r },
        Geom2D::Arc {
            c: c0,
            r,
            start_angle,
            end_angle,
            ccw,
        } => Geom2D::Arc {
            c: rot(c0),
            r: *r,
            start_angle: *start_angle + a,
            end_angle: *end_angle + a,
            ccw: *ccw,
        },
        Geom2D::Polyline { pts, closed } => Geom2D::Polyline {
            pts: pts.iter().map(rot).collect(),
            closed: *closed,
        },
    }
}
