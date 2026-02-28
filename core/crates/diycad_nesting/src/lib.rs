use diycad_geom::Point2;

#[derive(Debug, Clone, PartialEq)]
pub struct NestingPlan {
    pub placements: Vec<Point2>,
}

pub fn build_empty_plan() -> NestingPlan {
    NestingPlan {
        placements: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_plan_has_no_placements() {
        let plan = build_empty_plan();
        assert!(plan.placements.is_empty());
    }
}
