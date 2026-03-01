use crate::model::{PartEval, PlacementRect};
use craftcad_serialize::{
    NestJob, PartPlacementStatus, PartPlacementStatusKind, Reason, ReasonCode,
};

pub fn pack_parts(
    job: &NestJob,
    parts: &[PartEval],
    statuses: &mut Vec<PartPlacementStatus>,
) -> craftcad_serialize::Result<(Vec<PlacementRect>, Vec<String>)> {
    let mut placements = vec![];
    let mut failure_codes = vec![];

    let mut sheets = vec![];
    for s in &job.sheet_defs {
        for _ in 0..s.quantity {
            sheets.push((s.width, s.height, 0.0f64, 0.0f64, 0.0f64)); // w,h,cursor_x,cursor_y,row_h
        }
    }

    'part: for p in parts {
        for (sheet_index, sheet) in sheets.iter_mut().enumerate() {
            let (sw, sh, cx, cy, row_h) = sheet;
            let mut try_dims = vec![(p.width, p.height, 0.0)];
            if p.allow_rotate {
                try_dims.push((p.height, p.width, 90.0));
            }
            for (w, h, rot) in try_dims {
                if w > *sw || h > *sh {
                    continue;
                }
                if *cx + w > *sw {
                    *cx = 0.0;
                    *cy += *row_h;
                    *row_h = 0.0;
                }
                if *cy + h > *sh {
                    continue;
                }
                placements.push(PlacementRect {
                    part_id: p.part_id,
                    sheet_instance_index: sheet_index as u32,
                    x: *cx,
                    y: *cy,
                    rotation_deg: rot,
                    width: w,
                    height: h,
                });
                *cx += w;
                *row_h = row_h.max(h);
                statuses.push(PartPlacementStatus {
                    part_id: p.part_id,
                    status: PartPlacementStatusKind::Placed,
                    reason: None,
                });
                continue 'part;
            }
        }

        // reason classification
        let too_large = !job.sheet_defs.iter().any(|s| {
            (p.width <= s.width && p.height <= s.height)
                || (p.allow_rotate && p.height <= s.width && p.width <= s.height)
        });
        let code = if too_large {
            ReasonCode::NestPartTooLargeForAnySheet
        } else {
            ReasonCode::NestNoFeasiblePositionWithMarginAndKerf
        };
        failure_codes.push(code.as_str().to_string());
        statuses.push(PartPlacementStatus {
            part_id: p.part_id,
            status: PartPlacementStatusKind::Unplaced,
            reason: Some(Reason::from_code(code)),
        });
    }

    Ok((placements, failure_codes))
}
