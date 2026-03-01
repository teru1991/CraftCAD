use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_serialize::{Document, NestResultV1, NestTraceV1, Reason, ReasonCode, Result};
use diycad_geom::EpsilonPolicy;
use diycad_nesting::{run_nesting, RunLimits};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RunNestingInput {
    pub job_id: Uuid,
    pub eps: EpsilonPolicy,
    pub limits: RunLimits,
    pub doc_snapshot: Document,
}

pub struct RunNestingCommand {
    preview: Option<RunNestingDelta>,
}
impl RunNestingCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for RunNestingCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for RunNestingCommand {
    type Input = RunNestingInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        let before_job = input
            .doc_snapshot
            .jobs
            .iter()
            .find(|j| j.id == input.job_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
            .clone();
        let (result, trace) =
            run_nesting(&before_job, &input.doc_snapshot, &input.eps, input.limits)?;
        self.preview = Some(RunNestingDelta {
            job_id: input.job_id,
            before_result: before_job.result,
            before_trace: before_job.trace,
            after_result: Some(result),
            after_trace: Some(trace),
        });
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        Ok(Box::new(self.preview.clone().ok_or_else(|| {
            Reason::from_code(ReasonCode::CoreInvariantViolation)
        })?))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RunNestingDelta {
    job_id: Uuid,
    before_result: Option<NestResultV1>,
    before_trace: Option<NestTraceV1>,
    after_result: Option<NestResultV1>,
    after_trace: Option<NestTraceV1>,
}
impl Delta for RunNestingDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let j = doc
            .jobs
            .iter_mut()
            .find(|j| j.id == self.job_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        j.result = self.after_result.clone();
        j.trace = self.after_trace.clone();
        Ok(())
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        let j = doc
            .jobs
            .iter_mut()
            .find(|j| j.id == self.job_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        j.result = self.before_result.clone();
        j.trace = self.before_trace.clone();
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlacementPose {
    pub x: f64,
    pub y: f64,
    pub rotation_deg: f64,
}

#[derive(Debug, Clone)]
pub struct EditPlacementInput {
    pub job_id: Uuid,
    pub part_id: Uuid,
    pub sheet_index: i32,
    pub old_pose: PlacementPose,
    pub new_pose: PlacementPose,
}

pub struct EditPlacementCommand {
    preview: Option<EditPlacementDelta>,
}
impl EditPlacementCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for EditPlacementCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for EditPlacementCommand {
    type Input = EditPlacementInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        self.preview = Some(EditPlacementDelta {
            job_id: input.job_id,
            part_id: input.part_id,
            sheet_index: input.sheet_index,
            new_pose: input.new_pose,
            old_pose: Some(input.old_pose),
        });
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        Ok(Box::new(self.preview.clone().ok_or_else(|| {
            Reason::from_code(ReasonCode::CoreInvariantViolation)
        })?))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EditPlacementDelta {
    job_id: Uuid,
    part_id: Uuid,
    sheet_index: i32,
    new_pose: PlacementPose,
    old_pose: Option<PlacementPose>,
}
impl Delta for EditPlacementDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let job = doc
            .jobs
            .iter_mut()
            .find(|j| j.id == self.job_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        let res = job
            .result
            .as_mut()
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        let plc = res
            .placements
            .iter_mut()
            .find(|p| {
                p.part_id == self.part_id && p.sheet_instance_index == self.sheet_index as u32
            })
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        plc.x = self.new_pose.x;
        plc.y = self.new_pose.y;
        plc.rotation_deg = self.new_pose.rotation_deg;
        let w = plc.bbox.max_x - plc.bbox.min_x;
        let h = plc.bbox.max_y - plc.bbox.min_y;
        plc.bbox.min_x = plc.x;
        plc.bbox.min_y = plc.y;
        plc.bbox.max_x = plc.x + w;
        plc.bbox.max_y = plc.y + h;
        Ok(())
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        let old = self
            .old_pose
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        let job = doc
            .jobs
            .iter_mut()
            .find(|j| j.id == self.job_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        let res = job
            .result
            .as_mut()
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        let plc = res
            .placements
            .iter_mut()
            .find(|p| {
                p.part_id == self.part_id && p.sheet_instance_index == self.sheet_index as u32
            })
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        plc.x = old.x;
        plc.y = old.y;
        plc.rotation_deg = old.rotation_deg;
        let w = plc.bbox.max_x - plc.bbox.min_x;
        let h = plc.bbox.max_y - plc.bbox.min_y;
        plc.bbox.min_x = plc.x;
        plc.bbox.min_y = plc.y;
        plc.bbox.max_x = plc.x + w;
        plc.bbox.max_y = plc.y + h;
        Ok(())
    }
}
